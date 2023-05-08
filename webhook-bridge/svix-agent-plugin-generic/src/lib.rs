use std::time::{Duration, Instant};

use generic_queue::{
    rabbitmq::{
        BasicProperties, BasicPublishOptions, ConnectionProperties, RabbitMqBackend, RabbitMqConfig,
    },
    redis::{RedisConfig, RedisQueueBackend},
    sqs::{SqsConfig, SqsQueueBackend},
    Delivery, TaskQueueBackend, TaskQueueReceive,
};
use serde::{Deserialize, Serialize};
use svix::api::{MessageIn, PostOptions as PostOptions_, Svix};
use svix_agent_types::{async_trait, Plugin};

pub mod config;
pub use config::{
    GCPPubSubConsumerConfig, RabbitMqConsumerConfig, RabbitMqInputOpts, RedisConsumerConfig,
    RedisInputOpts, SqsConsumerConfig, SqsInputOpts,
};
mod error;
use error::Error;
mod gcp_pubsub;
pub use gcp_pubsub::GCPPubSubConsumerPlugin;

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");
pub const PLUGIN_VERS: &str = env!("CARGO_PKG_VERSION");

pub struct RabbitMqConsumerPlugin {
    input_options: RabbitMqInputOpts,
    svix_client: Svix,
}

pub struct RedisConsumerPlugin {
    input_options: RedisInputOpts,
    svix_client: Svix,
}

pub struct SqsConsumerPlugin {
    input_options: SqsInputOpts,
    svix_client: Svix,
}

impl TryInto<Box<dyn Plugin>> for RabbitMqConsumerConfig {
    type Error = &'static str;

    fn try_into(self) -> Result<Box<dyn Plugin>, Self::Error> {
        Ok(Box::new(RabbitMqConsumerPlugin::new(self)))
    }
}

impl TryInto<Box<dyn Plugin>> for RedisConsumerConfig {
    type Error = &'static str;

    fn try_into(self) -> Result<Box<dyn Plugin>, Self::Error> {
        Ok(Box::new(RedisConsumerPlugin::new(self)))
    }
}

impl TryInto<Box<dyn Plugin>> for SqsConsumerConfig {
    type Error = &'static str;

    fn try_into(self) -> Result<Box<dyn Plugin>, Self::Error> {
        Ok(Box::new(SqsConsumerPlugin::new(self)))
    }
}

impl RabbitMqConsumerPlugin {
    pub fn new(RabbitMqConsumerConfig { input, output }: RabbitMqConsumerConfig) -> Self {
        Self {
            input_options: input,
            svix_client: Svix::new(output.token, output.svix_options.map(Into::into)),
        }
    }

    async fn consume(&self) -> std::io::Result<()> {
        let mut consumer =
            <RabbitMqBackend as TaskQueueBackend<serde_json::Value>>::consuming_half(
                RabbitMqConfig {
                    uri: self.input_options.uri.clone(),
                    connection_properties: ConnectionProperties::default(),
                    publish_exchange: String::new(),
                    publish_routing_key: String::new(),
                    publish_options: BasicPublishOptions::default(),
                    publish_properites: BasicProperties::default(),
                    consume_queue: self.input_options.queue_name.clone(),
                    consumer_tag: self.input_options.consumer_tag.clone().unwrap_or_default(),
                    consume_options: self.input_options.consume_opts.unwrap_or_default(),
                    consume_arguments: self.input_options.consume_args.clone().unwrap_or_default(),
                    requeue_on_nack: self.input_options.requeue_on_nack,
                },
            )
            .await
            .map_err(Error::from)?;

        tracing::debug!("rabbitmq consuming: {}", &self.input_options.queue_name);

        // FIXME: `while let` swallows errors from `receive_all`.
        while let Ok(deliveries) = consumer.receive_all(1, Duration::from_millis(10)).await {
            let span = tracing::error_span!(
                "receive",
                otel.kind = "CONSUMER",
                messaging.system = "rabbitmq",
                messaging.operation = "receive",
                messaging.source = &self.input_options.queue_name,
                svixagent_plugin.name = PLUGIN_NAME,
                svixagent_plugin.vers = PLUGIN_VERS,
            );
            let _enter = span.enter();
            tracing::trace!("received: {}", deliveries.len());

            for delivery in deliveries {
                let span = tracing::error_span!("process", messaging.operation = "process");
                let _enter = span.enter();

                let payload = match Delivery::<serde_json::Value>::payload(&delivery) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("nack: {e}");
                        delivery.nack().await.map_err(Error::from)?;
                        continue;
                    }
                };

                match create_svix_message(&self.svix_client, payload).await {
                    Ok(_) => {
                        tracing::trace!("ack");
                        delivery.ack().await.map_err(Error::from)?
                    }

                    Err(e) => {
                        tracing::error!("nack: {e}");
                        delivery.nack().await.map_err(Error::from)?
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Plugin for RabbitMqConsumerPlugin {
    async fn run(&self) -> std::io::Result<()> {
        let mut fails: u64 = 0;
        let mut last_fail = Instant::now();

        tracing::info!("rabbitmq starting: {}", &self.input_options.queue_name);

        loop {
            if let Err(e) = self.consume().await {
                tracing::error!("{e}");
            }
            tracing::error!("rabbitmq disconnected: {}", &self.input_options.queue_name);

            if last_fail.elapsed() > Duration::from_secs(10) {
                // reset the fail count if we didn't have a hiccup in the past short while.
                tracing::trace!("been a while since last fail, resetting count");
                fails = 0;
            } else {
                fails += 1;
            }

            last_fail = Instant::now();
            tokio::time::sleep(Duration::from_millis((300 * fails).min(3000))).await;
        }
    }
}

impl RedisConsumerPlugin {
    pub fn new(RedisConsumerConfig { input, output }: RedisConsumerConfig) -> Self {
        Self {
            input_options: input,
            svix_client: Svix::new(output.token, output.svix_options.map(Into::into)),
        }
    }

    async fn consume(&self) -> std::io::Result<()> {
        let mut consumer =
            <RedisQueueBackend as TaskQueueBackend<CreateMessageRequest>>::consuming_half(
                RedisConfig {
                    dsn: self.input_options.dsn.clone(),
                    max_connections: self.input_options.max_connections,
                    reinsert_on_nack: self.input_options.reinsert_on_nack,
                    queue_key: self.input_options.queue_key.clone(),
                    consumer_group: self.input_options.consumer_group.clone(),
                    consumer_name: self.input_options.consumer_name.clone(),
                },
            )
            .await
            .map_err(Error::from)?;

        tracing::debug!("redis consuming: {}", &self.input_options.queue_key);
        // FIXME: `while let` swallows errors from `receive_all`.
        while let Ok(deliveries) = consumer.receive_all(1, Duration::from_millis(10)).await {
            let span = tracing::error_span!(
                "receive",
                otel.kind = "CONSUMER",
                messaging.system = "redis",
                messaging.operation = "receive",
                messaging.source = &self.input_options.queue_key,
                svixagent_plugin.name = PLUGIN_NAME,
                svixagent_plugin.vers = PLUGIN_VERS,
            );
            let _enter = span.enter();
            tracing::trace!("received: {}", deliveries.len());

            for delivery in deliveries {
                let span = tracing::error_span!("process", messaging.operation = "process");
                let _enter = span.enter();

                let payload = match Delivery::<serde_json::Value>::payload(&delivery) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("nack: {e}");
                        delivery.nack().await.map_err(Error::from)?;
                        continue;
                    }
                };

                match create_svix_message(&self.svix_client, payload).await {
                    Ok(_) => {
                        tracing::trace!("ack");
                        delivery.ack().await.map_err(Error::from)?
                    }
                    Err(e) => {
                        tracing::error!("nack: {e}");
                        delivery.nack().await.map_err(Error::from)?
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Plugin for RedisConsumerPlugin {
    async fn run(&self) -> std::io::Result<()> {
        let mut fails: u64 = 0;
        let mut last_fail = Instant::now();

        tracing::info!("redis starting: {}", &self.input_options.queue_key);

        loop {
            if let Err(e) = self.consume().await {
                tracing::error!("{e}");
            }

            tracing::error!("redis disconnected: {}", &self.input_options.queue_key);
            if last_fail.elapsed() > Duration::from_secs(10) {
                // reset the fail count if we didn't have a hiccup in the past short while.
                tracing::trace!("been a while since last fail, resetting count");
                fails = 0;
            } else {
                fails += 1;
            }

            last_fail = Instant::now();
            tokio::time::sleep(Duration::from_millis((300 * fails).min(3000))).await;
        }
    }
}

impl SqsConsumerPlugin {
    pub fn new(SqsConsumerConfig { input, output }: SqsConsumerConfig) -> Self {
        Self {
            input_options: input,
            svix_client: Svix::new(output.token, output.svix_options.map(Into::into)),
        }
    }

    async fn consume(&self) -> std::io::Result<()> {
        let mut consumer =
            <SqsQueueBackend as TaskQueueBackend<CreateMessageRequest>>::consuming_half(
                SqsConfig {
                    queue_dsn: self.input_options.queue_dsn.clone(),
                    override_endpoint: self.input_options.override_endpoint,
                },
            )
            .await
            .map_err(Error::from)?;

        tracing::debug!("sqs consuming: {}", &self.input_options.queue_dsn);
        // FIXME: `while let` swallows errors from `receive_all`.
        while let Ok(deliveries) = consumer.receive_all(1, Duration::from_millis(10)).await {
            let span = tracing::error_span!(
                "receive",
                otel.kind = "CONSUMER",
                messaging.system = "sqs",
                messaging.operation = "receive",
                messaging.source = &self.input_options.queue_dsn,
                svixagent_plugin.name = PLUGIN_NAME,
                svixagent_plugin.vers = PLUGIN_VERS,
            );
            let _enter = span.enter();
            tracing::trace!("received: {}", deliveries.len());

            for delivery in deliveries {
                let span = tracing::error_span!("process", messaging.operation = "process");
                let _enter = span.enter();

                let payload = match Delivery::<serde_json::Value>::payload(&delivery) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("nack: {e}");
                        delivery.nack().await.map_err(Error::from)?;
                        continue;
                    }
                };

                match create_svix_message(&self.svix_client, payload).await {
                    Ok(_) => {
                        tracing::trace!("ack");
                        delivery.ack().await.map_err(Error::from)?
                    }
                    Err(e) => {
                        tracing::error!("nack: {e}");
                        delivery.nack().await.map_err(Error::from)?
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Plugin for SqsConsumerPlugin {
    async fn run(&self) -> std::io::Result<()> {
        let mut fails: u64 = 0;
        let mut last_fail = Instant::now();

        tracing::info!("sqs starting: {}", &self.input_options.queue_dsn);

        loop {
            if let Err(e) = self.consume().await {
                tracing::error!("{e}");
            }

            tracing::error!("sqs disconnected: {}", &self.input_options.queue_dsn);

            if last_fail.elapsed() > Duration::from_secs(10) {
                // reset the fail count if we didn't have a hiccup in the past short while.
                tracing::trace!("been a while since last fail, resetting count");
                fails = 0;
            } else {
                fails += 1;
            }

            last_fail = Instant::now();
            tokio::time::sleep(Duration::from_millis((300 * fails).min(3000))).await;
        }
    }
}
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct PostOptions {
    idempotency_key: Option<String>,
}

impl From<PostOptions> for PostOptions_ {
    fn from(value: PostOptions) -> Self {
        PostOptions_ {
            idempotency_key: value.idempotency_key,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CreateMessageRequest {
    pub app_id: String,
    pub message: MessageIn,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_options: Option<PostOptions>,
}

async fn create_svix_message(svix: &Svix, value: serde_json::Value) -> std::io::Result<()> {
    let CreateMessageRequest {
        app_id,
        message,
        post_options,
    }: CreateMessageRequest = serde_json::from_value(value)?;
    let span = tracing::error_span!(
        "create_svix_message",
        app_id = app_id,
        event_type = message.event_type
    );
    let _enter = span.enter();

    svix.message()
        .create(app_id, message, post_options.map(Into::into))
        .await
        .map_err(Error::from)?;
    Ok(())
}