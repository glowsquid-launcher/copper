use anyhow::Result;
use structopt::StructOpt;

use crate::download_deps::download_deps;

pub async fn handle_args(args: Args) -> Result<()> {
    match args {
        Args::DownloadDependencies {
            root,
            version: version_id,
        } => download_deps(root, version_id).await?,
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
}
