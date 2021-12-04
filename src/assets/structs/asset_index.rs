use std::{collections::HashMap, error::Error, time::Duration};

use futures::future::join_all;
use indicatif::ProgressBar;
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};

use crate::util::create_download_task;

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, Object>,
}

impl AssetIndex {
    /// The save path should be /assets/objects
    pub async fn save_assets(&self, save_path: String) -> Result<(), Box<dyn Error>> {
        let client = ClientBuilder::new()
            .connection_verbose(true)
            .pool_idle_timeout(Some(Duration::from_secs(600)))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .build()
            .unwrap();

        let mut tasks = Vec::new();

        let pb = ProgressBar::new(0);

        // create a final path and return it along with the url
        let path_and_url: HashMap<String, String> = self
            .objects
            .iter()
            .map(|(_path, object)| {
                let url = format!(
                    "http://resources.download.minecraft.net/{}/{}",
                    &object.hash[..2],
                    object.hash
                );

                (format!("{}/{}", &object.hash[..2], object.hash), url)
            })
            .collect();

        // loop over the paths + urls
        for (path, url) in path_and_url.into_iter() {
            // because the path includes the file name, we need to remove the last part
            let full_path = format!("{}/{}", save_path, path);
            tasks.push(create_download_task(
                url,
                full_path,
                pb.clone(),
                client.clone(),
            ));
        }

        let amount_of_tasks = tasks.len();
        pb.set_length(amount_of_tasks.try_into().unwrap());

        // wait for all the tasks to finish

        join_all(tasks).await;

        println!("downloaded {} assets", amount_of_tasks);
        Ok(())
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
        let server_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

        let response = reqwest::get(server_url)
            .await
            .unwrap()
            .json::<LauncherMeta>()
            .await
            .unwrap();

        response
            .latest
            .version_for_release(&response)
            .version_manifest()
            .await
            .unwrap()
            .asset_index()
            .await
            .unwrap()
            .save_assets(
                std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
                    + "/tests-dir",
            )
            .await
            .unwrap();
    }
}
