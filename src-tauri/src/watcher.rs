// SPDX-License-Identifier: GPL-3.0-or-later
//! Debounced filesystem watcher. Emits `fs://changed` with the set of
//! directories that changed so the UI can refresh without re-listing everything.

use crate::state::AppState;
use notify::{RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

pub const CHANGED_EVENT: &str = "fs://changed";

#[tauri::command]
pub fn watch_start(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<(), String> {
    let mut slot = state.watcher.lock().unwrap_or_else(|p| p.into_inner());
    // Dropping the previous watcher closes its channel and retires its thread.
    *slot = None;

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if let Ok(event) = event {
            let _ = tx.send(event);
        }
    })
    .map_err(|e| e.to_string())?;

    for path in &paths {
        // Non-recursive: one watcher per open directory is enough for M1.
        let _ = watcher.watch(Path::new(path), RecursiveMode::NonRecursive);
    }
    *slot = Some(watcher);

    std::thread::Builder::new()
        .name("matrix-fs-watch".into())
        .spawn(move || {
            let mut pending: HashSet<PathBuf> = HashSet::new();
            loop {
                match rx.recv_timeout(Duration::from_millis(300)) {
                    Ok(event) => {
                        for path in event.paths {
                            if let Some(parent) = path.parent() {
                                pending.insert(parent.to_path_buf());
                            }
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        if !pending.is_empty() {
                            let dirs: Vec<String> = pending
                                .drain()
                                .map(|p| p.to_string_lossy().into_owned())
                                .collect();
                            let _ = app.emit(CHANGED_EVENT, dirs);
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}
