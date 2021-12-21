use super::version_manifest::VersionManifest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherMeta {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

impl LauncherMeta {
    pub async fn download_meta() -> Result<Self, Box<dyn Error>> {
        let server_url = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

        Ok(reqwest::get(server_url)
            .await
            .unwrap()
            .json::<LauncherMeta>()
            .await?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

impl Latest {
    pub fn version_for_release<'a>(&self, launcher_meta: &'a LauncherMeta) -> &'a Version {
        // get the latest release and find its version
        launcher_meta
            .versions
            .iter()
            .filter(|version| version.id == self.release)
            .next()
            .unwrap()
    }

    pub fn version_for_snapshot<'a>(&self, launcher_meta: &'a LauncherMeta) -> &'a Version {
        // get the latest snapshot and find its version
        launcher_meta
            .versions
            .iter()
            .filter(|version| version.id == self.snapshot)
            .next()
            .unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: Type,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

impl Version {
    pub async fn version_manifest(&self) -> Result<VersionManifest, Box<dyn Error>> {
        // download the version manifest and return a parsed version manifest
        Ok(reqwest::get(&self.url)
            .await?
            .json::<VersionManifest>()
            .await?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "old_alpha")]
    OldAlpha,
    #[serde(rename = "old_beta")]
    OldBeta,
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot")]
    Snapshot,
}

pub(crate) mod tests {
    #[tokio::test]
    pub async fn can_download_and_deserialize() {
        use super::LauncherMeta;
        use serde_json::Value;
        let server_url = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

        let response = reqwest::get(server_url)
            .await
            .unwrap()
            .json::<LauncherMeta>()
            .await
            .unwrap();

        let response_value = reqwest::get(server_url)
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();

        assert_eq!(response.latest.release, response_value["latest"]["release"]);
    }
}
