use crate::config::SenderInputOpts;
use crate::gcp_pubsub::GCPPubSubInputOpts;
use crate::rabbitmq::RabbitMqInputOpts;
use crate::redis::RedisInputOpts;
use crate::sqs::SqsInputOpts;
use crate::{run_inner, Consumer, Error};
use omniqueue::{
    backends,
    queue::{
        consumer::{DynConsumer, QueueConsumer},
        QueueBackend,
    },
};

use std::time::{Duration, Instant};
use svix_bridge_types::svix::api::Svix;
use svix_bridge_types::{async_trait, SenderInput, TransformationConfig, TransformerTx};
use tracing::instrument;

type Result<T> = std::result::Result<T, Error>;

pub struct QueueSender {
    name: String,
    source: String,
    system: String,
    input_opts: SenderInputOpts,
    transformation: Option<TransformationConfig>,
    transformer_tx: Option<TransformerTx>,
    svix_client: Svix,
}

async fn rabbit_consumer(cfg: &RabbitMqInputOpts) -> Result<DynConsumer> {
    backends::rabbitmq::RabbitMqBackend::builder(backends::rabbitmq::RabbitMqConfig {
        uri: cfg.uri.clone(),
        connection_properties: backends::rabbitmq::ConnectionProperties::default(),
        publish_exchange: String::new(),
        publish_routing_key: String::new(),
        publish_options: backends::rabbitmq::BasicPublishOptions::default(),
        publish_properites: backends::rabbitmq::BasicProperties::default(),
        consume_queue: cfg.queue_name.clone(),
        consumer_tag: cfg.consumer_tag.clone().unwrap_or_default(),
        consume_options: cfg.consume_opts.unwrap_or_default(),
        consume_arguments: cfg.consume_args.clone().unwrap_or_default(),
        requeue_on_nack: cfg.requeue_on_nack,
    })
    .make_dynamic()
    .build_consumer()
    .await
    .map_err(Error::from)
}

async fn redis_consumer(cfg: &RedisInputOpts) -> Result<DynConsumer> {
    backends::redis::RedisQueueBackend::<
        backends::redis::RedisMultiplexedConnectionManager,
    >::builder(backends::redis::RedisConfig {
        dsn: cfg.dsn.clone(),
        max_connections: cfg.max_connections,
        queue_key: cfg.queue_key.clone(),
        // consumer stuff we don't really care about
        reinsert_on_nack: false,
        consumer_group: String::new(),
        consumer_name: String::new(),
        payload_key: String::new(),
    })
        .make_dynamic()
        .build_consumer()
        .await
        .map_err(Error::from)
}

async fn sqs_consumer(cfg: &SqsInputOpts) -> Result<DynConsumer> {
    backends::sqs::SqsQueueBackend::builder(backends::sqs::SqsConfig {
        queue_dsn: cfg.queue_dsn.clone(),
        override_endpoint: cfg.override_endpoint,
    })
    .make_dynamic()
    .build_consumer()
    .await
    .map_err(Error::from)
}

pub async fn gcp_pupsub_consumer(cfg: &GCPPubSubInputOpts) -> Result<DynConsumer> {
    backends::gcp_pubsub::GcpPubSubBackend::builder(backends::gcp_pubsub::GcpPubSubConfig {
        subscription_id: cfg.subscription_id.clone(),
        credentials_file: cfg.credentials_file.clone(),
        // Don't need this. Topics are for producers only.
        topic_id: String::new(),
    })
    .make_dynamic()
    .build_consumer()
    .await
    .map_err(Error::from)
}

impl std::fmt::Debug for QueueSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SenderInput").finish()
    }
}

fn system_name(opts: &SenderInputOpts) -> &'static str {
    match opts {
        SenderInputOpts::GCPPubSub(_) => "gcp-pubsub",
        SenderInputOpts::RabbitMQ(_) => "rabbitmq",
        SenderInputOpts::Redis(_) => "redis",
        SenderInputOpts::SQS(_) => "sqs",
    }
}
impl QueueSender {
    pub fn new(name: String, opts: SenderInputOpts) -> Self {
        Self {
            name,
            source: "".to_string(),
            system: system_name(&opts).into(),
            input_opts: opts,
            transformation: None,
            transformer_tx: None,
            svix_client: (),
        }
    }
}

#[async_trait]
impl Consumer for QueueSender {
    fn source(&self) -> &str {
        &self.source
    }

    fn system(&self) -> &str {
        &self.system
    }

    fn transformer_tx(&self) -> Option<&TransformerTx> {
        self.transformer_tx.as_ref()
    }

    fn transformation(&self) -> Option<&TransformationConfig> {
        self.transformation.as_ref()
    }

    fn svix_client(&self) -> &Svix {
        &self.svix_client
    }

    async fn consumer(&self) -> std::io::Result<DynConsumer> {
        Ok(match &self.input_opts {
            SenderInputOpts::GCPPubSub(cfg) => gcp_pupsub_consumer(cfg).await,
            SenderInputOpts::RabbitMQ(cfg) => rabbit_consumer(cfg).await,
            SenderInputOpts::Redis(cfg) => redis_consumer(cfg).await,
            SenderInputOpts::SQS(cfg) => sqs_consumer(cfg).await,
        }
        .map_err(Error::from)?)
    }
}

#[async_trait]
impl SenderInput for QueueSender {
    fn name(&self) -> &str {
        &self.name
    }
    fn set_transformer(&mut self, tx: Option<TransformerTx>) {
        self.transformer_tx = tx;
    }
    async fn run(&self) -> std::io::Result<()> {
        run_inner(self).await
    }
}
