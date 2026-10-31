// SPDX-License-Identifier: GPL-3.0-or-later
//! Tauri commands: the entire surface the webview is allowed to call.

use crate::state::AppState;
use matrix_aggregate::TreeStats;
use matrix_core::{
    list_dir as core_list_dir, places as core_places, DirPage, ListOptions, Place, SortKey,
};
use matrix_thumbs::ThumbRef;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::Ordering;
use tauri::State;

fn sort_key(name: &str) -> SortKey {
    match name {
        "size" => SortKey::Size,
        "modified" => SortKey::Modified,
        "type" => SortKey::Type,
        _ => SortKey::Name,
    }
}

#[tauri::command]
pub fn list_dir(
    path: String,
    offset: u32,
    limit: u32,
    sort: String,
    descending: bool,
    dirs_first: bool,
    show_hidden: bool,
    filter: Option<String>,
) -> Result<DirPage, String> {
    let opts = ListOptions {
        sort: sort_key(&sort),
        descending,
        dirs_first,
        show_hidden,
        filter,
        offset,
        limit: limit.clamp(1, 5000),
    };
    core_list_dir(Path::new(&path), &opts).map_err(|e| format!("{path}: {e}"))
}

#[tauri::command]
pub async fn tree_stats(
    state: State<'_, AppState>,
    path: String,
    depth: u32,
) -> Result<TreeStats, String> {
    // Any newer request invalidates this one.
    let generation = state.tree_generation.fetch_add(1, Ordering::SeqCst) + 1;
    let shared = state.tree_generation.clone();
    let cancelled = move || shared.load(Ordering::SeqCst) != generation;

    tauri::async_runtime::spawn_blocking(move || {
        matrix_aggregate::tree_stats(Path::new(&path), depth.clamp(1, 10), cancelled)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn thumbnails(
    state: State<'_, AppState>,
    paths: Vec<String>,
    px: u32,
) -> Result<Vec<ThumbRef>, String> {
    let thumb = state.thumb.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let thumb = thumb
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        paths
            .iter()
            .filter_map(|path| thumb.thumbnail(Path::new(path), px).ok())
            .collect::<Vec<ThumbRef>>()
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn places() -> Vec<Place> {
    core_places()
}

#[tauri::command]
pub fn reveal(path: String) -> Result<(), String> {
    let target = Path::new(&path);
    let dir = if target.is_dir() {
        target
    } else {
        target.parent().unwrap_or(target)
    };
    Command::new("xdg-open")
        .arg(dir)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(&path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn preview_text(path: String, max_bytes: u32) -> Result<String, String> {
    let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    file.take(max_bytes.clamp(1, 1_048_576) as u64)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

#[tauri::command]
pub fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
}
