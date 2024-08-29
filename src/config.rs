use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    wallust_backend: wallust::backends::Backend,
    wallust_cache_path: PathBuf,
}
