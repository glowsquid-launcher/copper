use std::{collections::HashMap, error::Error, fs::create_dir_all, io::Write};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, Object>,
}

impl AssetIndex {
    /// The save path should be /assets/objects
    pub async fn save_assets(&self, save_path: String) -> Result<(), Box<dyn Error>> {
        let mut tasks = Vec::new();
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

        for (path, url) in path_and_url.into_iter() {
            let save_path_clone = save_path.clone();
            let mut path_without_last_vec = path.split("/").collect::<Vec<&str>>();
            path_without_last_vec.pop();
            let path_without_last = path_without_last_vec.join("/");

            tasks.push(tokio::spawn(async move {
                let mut response = reqwest::get(url).await.unwrap().bytes().await.unwrap();
                create_dir_all(format!("{}/{}", &save_path_clone, &path_without_last)).unwrap();
                let mut file =
                    std::fs::File::create(format!("{}/{}", save_path_clone, &path)).unwrap();
                file.write(&mut response).unwrap();
            }))
        }

        for task_handle in tasks {
            task_handle.await.unwrap()
        }

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
    async fn test_saving() {
        use crate::assets::structs::launcher_meta::LauncherMeta;
        let server_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

        println!(
            "{}",
            std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .to_string()
        );

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
