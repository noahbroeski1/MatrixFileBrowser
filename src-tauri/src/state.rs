// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared application state for the Tauri shell.

use matrix_thumbs::Thumbnailer;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

pub struct AppState {
    /// One thumbnailer shared across commands; it is cheap but owns cache paths.
    pub thumb: Arc<Mutex<Thumbnailer>>,
    /// Monotonic generation counter used to cancel superseded treemap walks.
    pub tree_generation: Arc<AtomicU64>,
    /// The live filesystem watcher, replaced whenever `watch_start` runs.
    pub watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
}

fn cache_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    base.join("matrixfilebrowser").join("thumbs")
}

impl AppState {
    pub fn new() -> Self {
        Self {
            thumb: Arc::new(Mutex::new(Thumbnailer::new(cache_dir()))),
            tree_generation: Arc::new(AtomicU64::new(0)),
            watcher: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
