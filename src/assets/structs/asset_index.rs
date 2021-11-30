use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, Object>,
}

impl AssetIndex {
    pub async fn save_assets(save_path: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub hash: String,
    pub size: i64,
}
