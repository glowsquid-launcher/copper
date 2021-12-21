use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, error::Error};

use futures::stream::FuturesUnordered;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::watch::{self, Sender};
use tokio::task;

use crate::util::{
    create_client, create_download_task, DownloadProgress, DownloadWatcher, FunkyFuturesThing,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, Object>,
}

impl AssetIndex {
    pub async fn save_index(&self, save_path: String) -> Result<(), Box<dyn Error>> {
        // serialize the struct to a json string
        let json = serde_json::to_string(self)?;

        // create file and save it
        let mut file = std::fs::File::create(save_path)?;
        file.write(json.as_bytes())?;

        Ok(())
    }

    /// The save path should be /assets/objects
    pub async fn download_assets(&self, save_path: PathBuf) -> FunkyFuturesThing {
        let client = create_client();

        let tasks = FuturesUnordered::new();
        // create a final path and return it along with the url
        let path_and_url: HashMap<String, String> = self
            .objects
            .iter()
            .map(|(_path, object)| {
                let url = format!(
                    "https://resources.download.minecraft.net/{}/{}",
                    &object.hash[..2],
                    object.hash
                );

                (format!("{}/{}", &object.hash[..2], object.hash), url)
            })
            .collect();

        // loop over the paths + urls
        for (path, url) in path_and_url.into_iter() {
            // because the path includes the file name, we need to remove the last part
            let full_path = save_path.join(path);
            tasks.push(create_download_task(url, full_path, Some(client.clone())));
        }

        tasks
    }

    async fn run_downloads(
        mut tasks: FunkyFuturesThing,
        progress_sender: Sender<DownloadProgress>,
    ) {
        let total = tasks.len();
        let mut finished = 0;

        while let Some(_) = tasks.next().await {
            finished += 1;
            let _ = progress_sender.send(DownloadProgress {
                total_size: total as u64,
                finished,
            });
        }
    }

    pub async fn start_download_assets(&self, save_path: PathBuf) -> DownloadWatcher {
        let (progress_sender, progress_receiver) = watch::channel(DownloadProgress {
            finished: 0,
            total_size: 0,
        });

        let tasks = self.download_assets(save_path).await;
        let download_task = task::spawn(Self::run_downloads(tasks, progress_sender));

        DownloadWatcher {
            progress_watcher: progress_receiver,
            download_task,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub hash: String,
    pub size: i64,
}

mod tests {
    #[tokio::test]
    async fn test_saving_assets() {
        use crate::assets::structs::launcher_meta::LauncherMeta;
        let server_url = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

        let response = reqwest::get(server_url)
            .await
            .unwrap()
            .json::<LauncherMeta>()
            .await
            .unwrap();

        let path = std::env::current_dir()
            .unwrap()
            .join("tests-dir")
            .join("assets");

        let mut watcher = response
            .latest
            .version_for_release(&response)
            .version_manifest()
            .await
            .unwrap()
            .asset_index()
            .await
            .unwrap()
            .start_download_assets(path)
            .await;

        while let Ok(_) = watcher.progress_watcher.changed().await {
            let progress = *watcher.progress_watcher.borrow();
            println!("{}/{}", progress.finished, progress.total_size); //derive copy on the DownloadProgress
        }
        let _ = watcher.download_task.await;
    }
}
