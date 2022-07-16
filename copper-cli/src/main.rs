pub mod cli;
pub mod download_deps;
pub mod launch_minecraft;

use anyhow::Result;
use clap::Parser;
use cli::{handle_args, Args};
use tracing_subscriber::EnvFilter;


#[tokio::main]
async fn main() -> Result<()> {
    setup_logger();
    let args = Args::try_parse()?;
    handle_args(args).await
}

fn setup_logger() {
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
