mod cache;

use cache::LocalCache;

#[tauri::command]
fn api_base_url() -> String {
    std::env::var("VITE_API_BASE_URL")
        .or_else(|_| std::env::var("HEALTHII_API_BASE_URL"))
        .unwrap_or_else(|_| "http://localhost:8080".into())
}

#[tauri::command]
fn cache_get(app: tauri::AppHandle, key: String) -> Result<Option<String>, String> {
    LocalCache::new(&app)?.get(&key)
}

#[tauri::command]
fn cache_set(app: tauri::AppHandle, key: String, value: String) -> Result<(), String> {
    LocalCache::new(&app)?.set(&key, &value)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![api_base_url, cache_get, cache_set])
        .setup(|app| {
            LocalCache::new(app.handle()).map_err(std::io::Error::other)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Healthii");
}
