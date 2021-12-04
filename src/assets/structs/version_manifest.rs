use std::{error::Error, io::Write, path::PathBuf, time::Duration};

use futures::future::join_all;
use indicatif::ProgressBar;
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};

use crate::util::create_download_task;

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub arguments: Arguments,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: i64,
    pub downloads: VersionInfoDownloads,
    pub id: String,
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,
    pub logging: Logging,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: i64,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub version_info_type: String,
}

impl VersionManifest {
    pub fn save_manifest_json(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        // serialize the struct to a json string
        let json = serde_json::to_string(self)?;

        // create file and save it
        let mut file = std::fs::File::create(save_path)?;
        file.write(json.as_bytes())?;

        Ok(())
    }

    pub async fn asset_index(&self) -> Result<super::asset_index::AssetIndex, Box<dyn Error>> {
        // Get json and return it
        Ok(reqwest::get(&self.asset_index.url)
            .await?
            .json::<super::asset_index::AssetIndex>()
            .await?)
    }

    pub async fn download_libraries(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        let client = ClientBuilder::new()
            .connection_verbose(true)
            .pool_idle_timeout(Some(Duration::from_secs(600)))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .build()
            .unwrap();

        let mut tasks = Vec::new();
        let pb = ProgressBar::new(0);

        for library in &self.libraries {
            // Check rules for the library to see if it should be downloaded
            if library.rules.is_some() {
                let rules = library.rules.as_ref().unwrap();

                // if the rules are not satisfied, skip the library
                if !VersionManifest::check_rules(rules) {
                    continue;
                }
            }

            // if we get here, then the library is allowed to be downloaded
            let url = library.downloads.artifact.url.clone();
            let subpath = library.downloads.artifact.path.as_ref().unwrap();

            // The full path includes the file name
            let full_path = PathBuf::from(save_path)
                .join(subpath)
                .to_string_lossy()
                .to_string();

            tasks.push(create_download_task(
                url,
                full_path,
                Some(pb.clone()),
                Some(client.clone()),
            ))
        }

        // wait for all the tasks to finish
        let amount_of_tasks = tasks.len();
        pb.set_length(amount_of_tasks.try_into().unwrap());

        join_all(tasks).await;

        Ok(())
    }

    pub async fn download_client_jar(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        let url = self.downloads.client.url.clone();
        let task = tokio::spawn(create_download_task(url, save_path.to_string(), None, None));
        task.await??;

        Ok(())
    }

    pub async fn download_server_jar(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        let url = self.downloads.server.url.clone();
        let task = tokio::spawn(create_download_task(url, save_path.to_string(), None, None));
        task.await??;

        Ok(())
    }

    fn check_rules(rules: &Vec<LibraryRule>) -> bool {
        for rule in rules {
            match rule.action {
                Action::Allow => {
                    if let Some(os) = &rule.os {
                        if match os.name {
                            Name::Linux => cfg!(target_os = "linux"),
                            Name::Osx => cfg!(target_os = "macos"),
                        } {
                            // continue going through the rules
                            continue;
                        } else {
                            // continue the loop because this library is not for this OS
                            return false;
                        }
                    } else {
                        // continue as this rule does not have an OS
                        continue;
                    }
                }
                Action::Disallow => {
                    if let Some(os) = &rule.os {
                        if match os.name {
                            Name::Linux => cfg!(target_os = "linux"),
                            Name::Osx => cfg!(target_os = "macos"),
                        } {
                            return false;
                        } else {
                            // continue going through the rules
                            continue;
                        }
                    } else {
                        // continue the loop because this library is not allowed for any OS
                        // (mojank moment)
                        return false;
                    }
                }
            }
        }

        true
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<GameElement>,
    pub jvm: Vec<JvmElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameClass {
    pub rules: Vec<GameRule>,
    pub value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameRule {
    pub action: Action,
    pub features: Features,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    pub is_demo_user: Option<bool>,
    pub has_custom_resolution: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JvmClass {
    pub rules: Vec<JvmRule>,
    pub value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JvmRule {
    pub action: Action,
    pub os: PurpleOs,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurpleOs {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    #[serde(rename = "totalSize")]
    pub total_size: Option<i64>,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfoDownloads {
    pub client: MappingsClass,
    pub client_mappings: MappingsClass,
    pub server: MappingsClass,
    pub server_mappings: MappingsClass,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MappingsClass {
    pub sha1: String,
    pub size: i64,
    pub url: String,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<LibraryRule>>,
    pub natives: Option<Natives>,
    pub extract: Option<Extract>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: MappingsClass,
    pub classifiers: Option<Classifiers>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Classifiers {
    pub javadoc: Option<MappingsClass>,
    #[serde(rename = "natives-linux")]
    pub natives_linux: Option<MappingsClass>,
    #[serde(rename = "natives-macos")]
    pub natives_macos: Option<MappingsClass>,
    #[serde(rename = "natives-windows")]
    pub natives_windows: Option<MappingsClass>,
    pub sources: Option<MappingsClass>,
    #[serde(rename = "natives-osx")]
    pub natives_osx: Option<MappingsClass>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extract {
    pub exclude: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Natives {
    pub osx: Option<String>,
    pub linux: Option<String>,
    pub windows: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryRule {
    pub action: Action,
    pub os: Option<FluffyOs>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FluffyOs {
    pub name: Name,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingClient {
    pub argument: String,
    pub file: AssetIndex,
    #[serde(rename = "type")]
    pub client_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GameElement {
    GameClass(GameClass),
    String(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JvmElement {
    JvmClass(JvmClass),
    String(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "disallow")]
    Disallow,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Name {
    #[serde(rename = "osx")]
    Osx,
    #[serde(rename = "linux")]
    Linux,
}

mod tests {
    #[tokio::test]
    async fn can_download_and_deserialize() {
        use crate::assets::structs::version_manifest::VersionManifest;
        use serde_json::Value;
        let server_url = "https://launchermeta.mojang.com/v1/packages/59734133c4768dd79fa3c9b7a7650a713a8d294a/1.17.1.json";

        let response = reqwest::get(server_url)
            .await
            .unwrap()
            .json::<VersionManifest>()
            .await
            .unwrap();

        let response_value = reqwest::get(server_url)
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();

        assert!(response.id == response_value["id"]);
    }

    #[tokio::test]
    async fn can_save_libraries() {
        use crate::assets::structs::version_manifest::VersionManifest;
        let server_url = "https://launchermeta.mojang.com/v1/packages/59734133c4768dd79fa3c9b7a7650a713a8d294a/1.17.1.json";

        let response = reqwest::get(server_url)
            .await
            .unwrap()
            .json::<VersionManifest>()
            .await
            .unwrap();

        response
            .download_libraries(
                &(std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
                    + "/tests-dir"),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn can_save() {
        use crate::assets::structs::version_manifest::VersionManifest;
        let server_url = "https://launchermeta.mojang.com/v1/packages/59734133c4768dd79fa3c9b7a7650a713a8d294a/1.17.1.json";

        let response = reqwest::get(server_url)
            .await
            .unwrap()
            .json::<VersionManifest>()
            .await
            .unwrap();

        response
            .save_manifest_json(
                &(std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
                    + "/tests-dir/test.json"),
            )
            .unwrap();
    }
}
