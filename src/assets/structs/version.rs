use std::{error::Error, fs::create_dir_all, io::Write, path::PathBuf};

use futures::{stream::FuturesUnordered, StreamExt};
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::watch::{self, Sender},
    task,
};

use crate::util::{
    create_client, create_download_task, DownloadProgress, DownloadWatcher, ListOfResultHandles,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub arguments: Option<Arguments>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,
    pub assets: Option<String>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<i64>,
    pub downloads: Option<VersionInfoDownloads>,
    pub id: Option<String>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    pub libraries: Option<Vec<Library>>,
    pub logging: Option<Logging>,
    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: Option<i64>,
    #[serde(rename = "releaseTime")]
    pub release_time: Option<String>,
    pub time: Option<String>,
    #[serde(rename = "type")]
    pub version_info_type: Option<String>,
}

impl Version {
    pub fn merge(&self, lower: Self) -> Self {
        todo!()
    }

    pub fn save_json(&self, save_path: PathBuf) -> Result<(), Box<dyn Error>> {
        trace!("Saving version to {}", save_path.display());
        // serialize the struct to a json string
        let json = serde_json::to_string(self)?;
        create_dir_all(save_path.parent().ok_or(
            "save_path doesnt have a parent. Make sure you include the {version}.json bit at the end",
        )?)?;

        trace!("Creating file at {}", save_path.display());
        let mut file = std::fs::File::create(&save_path)?;
        trace!("Writing version file to file");
        file.write(json.as_bytes())?;

        debug!("Saved version file to {}", &save_path.display());
        Ok(())
    }

    pub async fn asset_index(&self) -> Result<super::asset_index::AssetIndex, Box<dyn Error>> {
        trace!("Downloading asset index");
        // Get json and return it
        Ok(reqwest::get(
            &self
                .asset_index
                .as_ref()
                .ok_or("No asset index provided")?
                .url,
        )
        .await?
        .json::<super::asset_index::AssetIndex>()
        .await?)
    }

    pub async fn download_libraries(
        &self,
        save_path: PathBuf,
    ) -> Result<ListOfResultHandles, Box<dyn Error>> {
        trace!("Downloading libraries");
        let client = create_client();

        let tasks = FuturesUnordered::new();

        for library in self.libraries.as_ref().ok_or("No libraries provided")? {
            // Check rules for the library to see if it should be downloaded
            if let Some(rules) = &library.rules {
                trace!("Library {} has rules, checking them", library.name);
                // if the rules are not satisfied, skip the library
                if !Version::check_library_rules(rules) {
                    continue;
                }
            }
            trace!(
                "Library {} has no rules or the rules passed, downloading",
                library.name
            );

            // if we get here, then the library is allowed to be downloaded

            Self::create_save_task(
                &library.downloads.artifact,
                &save_path,
                library,
                &tasks,
                &client,
            );

            if let Some(classifiers) = &library.downloads.classifiers {
                match std::env::consts::OS {
                    "windows" => {
                        if let Some(windows) = &classifiers.natives_windows {
                            Self::create_save_task(windows, &save_path, library, &tasks, &client);
                        } else {
                            continue;
                        }
                    }
                    "macos" => {
                        if let Some(macos) = &classifiers.natives_macos {
                            Self::create_save_task(macos, &save_path, library, &tasks, &client);
                        } else if let Some(osx) = &classifiers.natives_osx {
                            Self::create_save_task(osx, &save_path, library, &tasks, &client);
                        } else {
                            continue;
                        }
                    }
                    "linux" => {
                        if let Some(linux) = &classifiers.natives_linux {
                            Self::create_save_task(linux, &save_path, library, &tasks, &client);
                        } else {
                            continue;
                        }
                    }
                    _ => panic!("Unsupported OS"),
                };
            }
        }

        debug!("Created {} library download tasks", tasks.len());
        Ok(tasks)
    }

    async fn run_downloads(
        mut tasks: ListOfResultHandles,
        progress_sender: Sender<DownloadProgress>,
    ) {
        trace!("Running library download tasks");
        let total = tasks.len();
        let mut finished = 0;

        while let Some(_) = tasks.next().await {
            finished += 1;
            debug!("{}/{} library downloads finished", finished, total);
            let _ = progress_sender.send(DownloadProgress {
                total_size: total as u64,
                finished,
            });
        }
    }

    pub async fn start_download_libraries(
        &self,
        save_path: PathBuf,
    ) -> Result<DownloadWatcher, Box<dyn Error>> {
        trace!("Starting download libraries");
        trace!("Creating progress watcher");
        let (progress_sender, progress_receiver) = watch::channel(DownloadProgress {
            finished: 0,
            total_size: 0,
        });

        trace!("Creating download tasks");
        let tasks = self.download_libraries(save_path).await?;
        trace!("Starting download tasks");
        let download_task = task::spawn(Self::run_downloads(tasks, progress_sender));

        Ok(DownloadWatcher {
            progress_watcher: progress_receiver,
            download_task,
        })
    }

    pub async fn download_client_jar(&self, save_path: PathBuf) -> Result<(), Box<dyn Error>> {
        let url = self
            .downloads
            .as_ref()
            .ok_or("downloads were not provided")?
            .client
            .url
            .clone();
        let task = tokio::spawn(create_download_task(url, save_path, None));

        // the ultimate jank
        if let Err(e) = task.await?? {
            return Err(e);
        };

        Ok(())
    }

    pub async fn download_server_jar(&self, save_path: PathBuf) -> Result<(), Box<dyn Error>> {
        let url = self
            .downloads
            .as_ref()
            .ok_or("downloads were not provided")?
            .server
            .url
            .clone();
        let task = tokio::spawn(create_download_task(url, save_path, None));

        // the ultimate jank
        if let Err(e) = task.await?? {
            return Err(e);
        };

        Ok(())
    }

    pub fn check_library_rules(rules: &Vec<LibraryRule>) -> bool {
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

    fn create_save_task(
        mappings_class: &MappingsClass,
        save_path: &PathBuf,
        library: &Library,
        tasks: &FuturesUnordered<task::JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>>,
        client: &reqwest::Client,
    ) {
        let url = mappings_class.url.clone();
        let sub_path = mappings_class
        .path
        .as_ref()
        .expect("library doesnt have a path. Please report this bug to https://github.com/glowsquid-launcher/glowsquid/issues");

        let full_path = save_path.join(sub_path);
        debug!(
            "Creating download task for library {}, saving to {}",
            library.name,
            full_path.display()
        );
        tasks.push(create_download_task(url, full_path, Some(client.clone())));
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