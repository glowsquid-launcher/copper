use std::path::PathBuf;

use anyhow::Result;
use structopt::StructOpt;

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
        #[structopt(short, long)]
        root: String,
        /// The minecraft version.
        ///
        /// This can be any minecraft version (including snapshot versions) and can be "latest" for the latest release
        #[structopt(short, long)]
        version: String,
    },
    /// Launch minecraft
    Launch {
        /// The root .minecraft folder.
        #[structopt(short, long, parse(from_os_str))]
        root: PathBuf,
        /// The minecraft version.
        ///
        /// This can be any minecraft version (including snapshot versions) and can be "latest" for the latest release
        #[structopt(short, long)]
        version: String,

        /// Your access token.
        #[structopt(short, long)]
        access_token: String,

        #[structopt(short, long)]
        username: String,

        #[structopt(short = "id", long)]
        uuid: String,

        #[structopt(short, long)]
        xbox_uid: String,
    },
}
