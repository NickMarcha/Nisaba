mod commands;
mod indexer;
mod watcher;

use std::sync::{Arc, Mutex};
use std::sync::mpsc;

pub struct VaultWatcherState {
    pub stop_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(VaultWatcherState {
            stop_tx: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            commands::open_vault,
            commands::list_vault_files,
            commands::read_file,
            commands::write_file,
            commands::rename_file,
            commands::index_vault,
            commands::query_blocks,
            commands::query_links,
            commands::query_files,
            commands::watch_vault,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
