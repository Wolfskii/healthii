//! Local JSON cache for a future sync engine. This is intentionally small.

use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

pub struct LocalCache {
    path: PathBuf,
}

impl LocalCache {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;
        fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        Ok(Self {
            path: dir.join("local-cache.json"),
        })
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self
            .read()?
            .get(key)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned))
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), String> {
        if key.is_empty() || key.len() > 120 {
            return Err("invalid cache key".into());
        }
        let mut data = self.read()?;
        data[key] = json!(value);
        fs::write(
            &self.path,
            serde_json::to_vec_pretty(&data).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())
    }

    fn read(&self) -> Result<Value, String> {
        if !self.path.exists() {
            return Ok(json!({}));
        }
        let bytes = fs::read(&self.path).map_err(|error| error.to_string())?;
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())
    }
}
