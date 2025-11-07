use anyhow::Result;
use clap::Parser;
use rush::{cli, config, runtime};

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = fmt().with_env_filter(env_filter).try_init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = cli::Cli::parse();
    let config = config::Config::from_cli(cli)?;

    runtime::run(config).await
}
