use std::{error::Error, path::PathBuf};
use structopt::StructOpt;

use minecraft_rs::assets::structs::launcher_meta::LauncherMeta;
use minecraft_rs::util::DivPathBuf;

pub async fn handle_args(args: Args) -> Result<(), Box<dyn Error>> {
    match args {
        Args::DownloadDependencies {
            root,
            version: version_id,
        } => {
            let launcher_meta = LauncherMeta::download_meta().await?;

            let version_info = if version_id == "latest" {
                launcher_meta.latest.version_for_release(&launcher_meta)
            } else {
                launcher_meta
                    .versions
                    .iter()
                    .find(|version| version.id == version_id)
                    .unwrap()
            };

            let version = version_info.version_manifest().await?;

            let root_path = DivPathBuf(PathBuf::from(root));
            let libraries_path = &root_path / "libraries";
            let version_path = &root_path / "versions" / &*version.id;

            let mut libraries_watcher = version
                .start_download_libraries(libraries_path.to_path_buf())
                .await;

            while let Ok(_) = libraries_watcher.progress_watcher.changed().await {
                let progress = *libraries_watcher.progress_watcher.borrow();
                println!(
                    "libraries downloaded: {}/{}",
                    progress.finished, progress.total_size
                ); //derive copy on the DownloadProgress
            }

            libraries_watcher.download_task.await?;

            let mut asset_watcher = version
                .asset_index()
                .await?
                .start_download_assets((&root_path / "assets" / "objects").to_path_buf())
                .await;

            while let Ok(_) = asset_watcher.progress_watcher.changed().await {
                let progress = *asset_watcher.progress_watcher.borrow();
                println!(
                    "assets downloaded: {}/{}",
                    progress.finished, progress.total_size
                ); //derive copy on the DownloadProgress
            }

            asset_watcher.download_task.await?;

            version.save_manifest_json(
                (&version_path / &*format!("{}.json", &*version.id)).to_path_buf(),
            )?;

            println!("saved manifest");

            version
                .download_client_jar(
                    (&version_path / &*format!("{}.jar", &*version.id)).to_path_buf(),
                )
                .await?;

            println!("saved client jar");
        }
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
pub enum Args {
    DownloadDependencies {
        /// The .minecraft folder
        #[structopt(short)]
        root: String,
        /// the minecraft version
        #[structopt(short)]
        version: String,
    },
}
