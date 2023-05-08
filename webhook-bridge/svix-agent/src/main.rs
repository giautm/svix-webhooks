use self::config::Config;
use clap::Parser;
use lazy_static::lazy_static;
use opentelemetry::runtime::Tokio;
use opentelemetry_otlp::WithExportConfig;
use std::path::PathBuf;
use svix_agent_types::Plugin;
use svix_ksuid::{KsuidLike as _, KsuidMs};
use tracing_subscriber::prelude::*;

mod config;

lazy_static! {
    // Seems like it would be useful to be able to configure this.
    // In some docker setups, hostname is sometimes the container id, and advertising this can be
    // helpful.
    pub static ref INSTANCE_ID: String = KsuidMs::new(None, None).to_string();
}

fn get_svc_identifiers(cfg: &Config) -> opentelemetry::sdk::Resource {
    opentelemetry::sdk::Resource::new(vec![
        opentelemetry::KeyValue::new(
            "service.name",
            cfg.opentelemetry_service_name
                .as_deref()
                // FIXME: can we do something better?
                .unwrap_or("svix-agent")
                .to_owned(),
        ),
        opentelemetry::KeyValue::new("instance_id", INSTANCE_ID.to_owned()),
    ])
}

fn setup_tracing(cfg: &Config) {
    if std::env::var_os("RUST_LOG").is_none() {
        const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
        let level = cfg.log_level.to_string();
        let var = vec![
            format!("{CRATE_NAME}={level}"),
            // XXX: Assuming this applies to the Producer side (aka `og-ingester`) when we fold it back in.
            format!("tower_http={level}"),
        ];
        std::env::set_var("RUST_LOG", var.join(","));
    }

    let otel_layer = cfg.opentelemetry_address.as_ref().map(|addr| {
        // Configure the OpenTelemetry tracing layer
        opentelemetry::global::set_text_map_propagator(
            opentelemetry::sdk::propagation::TraceContextPropagator::new(),
        );

        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(addr);

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(
                opentelemetry::sdk::trace::config()
                    .with_sampler(
                        cfg.opentelemetry_sample_ratio
                            .map(opentelemetry::sdk::trace::Sampler::TraceIdRatioBased)
                            .unwrap_or(opentelemetry::sdk::trace::Sampler::AlwaysOn),
                    )
                    .with_resource(get_svc_identifiers(cfg)),
            )
            .install_batch(Tokio)
            .unwrap();

        tracing_opentelemetry::layer().with_tracer(tracer)
    });

    // Then initialize logging with an additional layer printing to stdout. This additional layer is
    // either formatted normally or in JSON format
    // Fails if the subscriber was already initialized, which we can safely and silently ignore.
    let _ = match cfg.log_format {
        config::LogFormat::Default => {
            let stdout_layer = tracing_subscriber::fmt::layer();
            tracing_subscriber::Registry::default()
                .with(otel_layer)
                .with(stdout_layer)
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .try_init()
        }
        config::LogFormat::Json => {
            let fmt = tracing_subscriber::fmt::format().json().flatten_event(true);
            let json_fields = tracing_subscriber::fmt::format::JsonFields::new();

            let stdout_layer = tracing_subscriber::fmt::layer()
                .event_format(fmt)
                .fmt_fields(json_fields);

            tracing_subscriber::Registry::default()
                .with(otel_layer)
                .with(stdout_layer)
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .try_init()
        }
    };
}

async fn supervise(consumers: Vec<Box<dyn Plugin>>) -> std::io::Result<()> {
    let mut set = tokio::task::JoinSet::new();
    for consumer in consumers {
        set.spawn(async move {
            // FIXME: needs much better signaling for termination
            loop {
                let fut = consumer.run();
                // If this future returns, the consumer terminated unexpectedly.
                if let Err(e) = fut.await {
                    tracing::warn!("plugin unexpectedly terminated: {}", e);
                } else {
                    tracing::warn!("plugin unexpectedly terminated");
                }
            }
        });
    }

    // FIXME: add signal handling to trigger a (intentional) graceful shutdown.

    // FIXME: when a plugin exits unexpectedly, what do?
    //   Most consumers are probably stateful/brittle and may disconnect from time to time.
    //   Ideally none of these tasks would ever return Ok or Err. They'd run forever.
    //   Having the tasks themselves try to recover means if we see a task finish here, something
    //   must be really wrong, so maybe we trigger a shutdown of the rest when one stops here.
    while let Some(_res) = set.join_next().await {
        // In order for plugins to coordinate a shutdown, maybe they could:
        // - have a shutdown method and handle their own internal signalling, or maybe
        // - take a oneshot channel as an arg to `run()`
        // Basically we need something that formalizes the shutdown flow in a cross-crate
        // friendly way.
        todo!("graceful shutdown");
    }
    Ok(())
}

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, env = "SVIX_AGENT_CFG")]
    cfg: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let config = args.cfg.unwrap_or_else(|| {
        std::env::current_dir()
            .expect("current dir")
            .join("svix-agent.yaml")
    });
    let cfg: Config = serde_yaml::from_str(&std::fs::read_to_string(&config).map_err(|e| {
        let p = config.into_os_string().into_string().expect("config path");
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read {p}: {e}"),
        )
    })?)
    .map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to parse config: {}", e),
        )
    })?;
    setup_tracing(&cfg);

    tracing::info!("starting");

    let mut consumers = Vec::with_capacity(cfg.plugins.len());
    for cc in cfg.plugins {
        let consumer = cc.try_into().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to configure consumer plugin: {}", e),
            )
        })?;
        consumers.push(consumer);
    }
    if consumers.is_empty() {
        tracing::warn!("No consumers configured.")
    }
    supervise(consumers).await?;
    tracing::info!("exiting...");
    Ok(())
}