pub mod cli;
pub mod download_deps;
pub mod launch_minecraft;

use anyhow::Result;
use clap::Parser;
use cli::{handle_args, Args};
use fern::colors::{Color, ColoredLevelConfig};

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    let args = Args::try_parse()?;
    handle_args(args).await
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let colours = ColoredLevelConfig::new()
                .info(Color::Green)
                .warn(Color::Yellow)
                .error(Color::Red)
                .debug(Color::BrightBlack)
                .trace(Color::BrightBlack);

            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                colours.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
