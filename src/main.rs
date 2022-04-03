use std::sync::Arc;

use clap::Parser;
use color_eyre::eyre;
use opentelemetry::global;
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

mod cli;
mod error;
mod http;
mod webdriver;

pub use error::Error;

pub struct State {
    pub driver: webdriver::ChromeDriver,
}

fn init_tracing(tracing_opts: cli::TracingOpts) -> eyre::Result<()> {
    if tracing_opts.enabled {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

        // Install a new OpenTelemetry trace pipeline
        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name(tracing_opts.service_name)
            .install_simple()?;

        // Create a tracing layer with the configured tracer
        let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let fmt_layer = tracing_subscriber::fmt::layer();
        let env_filter = EnvFilter::from_default_env();
        let collector = tracing_subscriber::Registry::default()
            .with(ErrorLayer::default())
            .with(opentelemetry)
            .with(env_filter)
            .with(fmt_layer);

        tracing::subscriber::set_global_default(collector)
            .expect("Unable to set a global collector");
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let opts = cli::Opts::parse();
    init_tracing(opts.tracing_opts)?;

    let state = Arc::new(State {
        driver: webdriver::ChromeDriver::new("/usr/bin/chromedriver", 4444)?,
    });

    http::start_server(state.clone()).await;

    error!("stopping application");

    Ok(())
}
