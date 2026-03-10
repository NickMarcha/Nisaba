// File system watcher for vault folder. Emits Tauri events when files change.

use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use tauri::{AppHandle, Emitter};

const VAULT_FILES_CHANGED: &str = "vault-files-changed";

/// Start watching a vault directory. Emits "vault-files-changed" when .md files change.
/// Stops any previous watcher. Pass stop_tx from app state to allow stopping when vault changes.
pub fn watch_vault(
    app: AppHandle,
    vault_path: &Path,
    stop_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
) -> Result<(), String> {
    if !vault_path.is_dir() {
        return Err("Path is not a directory".into());
    }

    // Stop previous watcher
    if let Ok(mut guard) = stop_tx.lock() {
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
        }
    }

    let (tx, rx) = mpsc::channel();
    *stop_tx.lock().map_err(|_| "lock poisoned")? = Some(tx);

    let app = app.clone();
    let vault_path = vault_path.to_path_buf();

    std::thread::spawn(move || {
        let mut debouncer = match new_debouncer(
            Duration::from_millis(300),
            move |res: DebounceEventResult| {
                match res {
                    Ok(events) => {
                        let has_md = events.iter().any(|e| {
                            e.path
                                .extension()
                                .map_or(false, |ext| ext.to_string_lossy() == "md")
                        });
                        if has_md {
                            let _ = app.emit(VAULT_FILES_CHANGED, ());
                        }
                    }
                    Err(_) => {}
                }
            },
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("watcher: failed to create debouncer: {}", e);
                return;
            }
        };

        use notify_debouncer_mini::notify::RecursiveMode;
        if let Err(e) = debouncer.watcher().watch(&vault_path, RecursiveMode::Recursive) {
            eprintln!("watcher: failed to watch {:?}: {}", vault_path, e);
            return;
        }

        // Keep debouncer alive until we receive stop signal
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
            }
        }
    });

    Ok(())
}
