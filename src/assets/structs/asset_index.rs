use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, Object>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub hash: String,
    pub size: i64,
}
