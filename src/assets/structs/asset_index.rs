use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, Object>,
}

impl AssetIndex {
    pub async fn save_assets(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        let mut tasks = Vec::new();
        let path_and_url: HashMap<String, String> = self
            .objects
            .iter()
            .map(|(path, object)| {
                let url = format!(
                    "http://resources.download.minecraft.net/{}/{}/",
                    &object.hash[..2],
                    object.hash
                );

                (path.clone(), url)
            })
            .collect();

        for (path, url) in path_and_url.iter() {
            tasks.push(tokio::spawn(async move { todo!() }))
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
            .save_assets("tests-dir")
            .await
            .unwrap();
    }
}
