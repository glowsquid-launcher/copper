use minecraft_rs::cli::{handle_args, Args};
use std::error::Error;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();

    handle_args(args).await
}
