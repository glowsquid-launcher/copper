use std::path::PathBuf;

use anyhow::Result;
use clap::StructOpt;

use crate::{download_deps::download_deps, launch_minecraft::launch_minecraft};

pub async fn handle_args(args: Args) -> Result<()> {
    match args {
        Args::DownloadDependencies {
            root,
            version: version_id,
        } => download_deps(root, version_id).await?,
        Args::Launch {
            root,
            version: version_id,
            access_token,
            username,
            uuid,
            xbox_uid,
        } => launch_minecraft(username, uuid, access_token, xbox_uid, root, version_id).await?,
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
pub enum Args {
    /// Download minecrafts dependencies.
    ///
    /// This includes assets, libraries, the client jar, the client manifest, the version manifest, and more
    DownloadDependencies {
        /// The root .minecraft folder.
        ///
        /// This is where everything will be downloaded to
        #[structopt(short, long, value_parser)]
        root: String,
        /// The minecraft version.
        ///
        /// This can be any minecraft version (including snapshot versions) and can be "latest" for the latest release
        #[structopt(short, long, value_parser)]
        version: String,
    },
    /// Launch minecraft
    Launch {
        /// The root .minecraft folder.
        #[structopt(short, long, value_parser)]
        root: PathBuf,
        /// The minecraft version.
        ///
        /// This can be any minecraft version (including snapshot versions) and can be "latest" for the latest release
        #[structopt(short, long, value_parser)]
        version: String,

        /// Your access token.
        #[structopt(short, long, value_parser)]
        access_token: String,

        #[structopt(short, long, value_parser)]
        username: String,

        #[structopt(short = 'i', long, value_parser)]
        uuid: String,

        #[structopt(short, long, value_parser)]
        xbox_uid: String,
    },
}
