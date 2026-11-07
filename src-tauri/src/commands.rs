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

#[allow(clippy::too_many_arguments)]
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

#[tauri::command]
pub fn rename_path(path: String, new_name: String) -> Result<String, String> {
    let src = std::path::Path::new(&path);
    let parent = src.parent().ok_or_else(|| "no parent".to_string())?;
    let new_name = new_name.trim();
    if new_name.is_empty() || new_name.contains('/') || new_name == "." || new_name == ".." {
        return Err("invalid name".to_string());
    }
    let dest = parent.join(new_name);
    if dest.exists() {
        return Err(format!("{} already exists", dest.display()));
    }
    std::fs::rename(src, &dest).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn move_items(paths: Vec<String>, dest_dir: String) -> Result<Vec<String>, String> {
    let dest = std::path::Path::new(&dest_dir);
    if !dest.is_dir() {
        return Err(format!("{} is not a directory", dest_dir));
    }
    let mut out = Vec::new();
    for path in paths {
        let src = std::path::Path::new(&path);
        let name = src.file_name().ok_or_else(|| format!("bad path {path}"))?;
        let target = dest.join(name);
        if target.exists() {
            return Err(format!(
                "{} already exists in destination",
                target.display()
            ));
        }
        // Try rename first (same filesystem); fall back to copy+remove.
        if std::fs::rename(src, &target).is_err() {
            if src.is_dir() {
                copy_dir_recursive(src, &target)?;
                std::fs::remove_dir_all(src).map_err(|e| e.to_string())?;
            } else {
                std::fs::copy(src, &target).map_err(|e| e.to_string())?;
                std::fs::remove_file(src).map_err(|e| e.to_string())?;
            }
        }
        out.push(target.to_string_lossy().into_owned());
    }
    Ok(out)
}

fn copy_dir_recursive(src: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn trash_items(paths: Vec<String>) -> Result<(), String> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })
        .ok_or_else(|| "no data home".to_string())?;
    let trash_files = data_home.join("Trash").join("files");
    let trash_info = data_home.join("Trash").join("info");
    std::fs::create_dir_all(&trash_files).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&trash_info).map_err(|e| e.to_string())?;

    let now = chrono_now();
    for path in paths {
        let src = std::path::Path::new(&path);
        let name = src
            .file_name()
            .ok_or_else(|| format!("bad path {path}"))?
            .to_string_lossy()
            .into_owned();
        // Find a unique name in trash.
        let mut target = trash_files.join(&name);
        let mut counter = 1;
        while target.exists() {
            let stem = src
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let ext = src
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            target = trash_files.join(format!("{stem}.{counter}{ext}"));
            counter += 1;
        }
        let final_name = target.file_name().unwrap().to_string_lossy().into_owned();

        // Write .trashinfo
        let info_path = trash_info.join(format!("{final_name}.trashinfo"));
        let info = format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            path.replace('\\', "\\\\").replace('\n', ""),
            now
        );
        std::fs::write(&info_path, info).map_err(|e| e.to_string())?;

        std::fs::rename(src, &target).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn chrono_now() -> String {
    // ISO 8601 without external crate: use system time.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Convert to UTC-ish YYYY-MM-DDTHH:MM:SS (approximate; ignores tz, acceptable for trashinfo)
    let days = secs / 86400;
    // Simple conversion: days since epoch to y/m/d
    let (y, m, d) = days_to_ymd(days);
    let rem = secs % 86400;
    let hh = rem / 3600;
    let mm = (rem % 3600) / 60;
    let ss = rem % 60;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", y, m, d, hh, mm, ss)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Convert days since 1970-01-01 to (year, month, day).
    let mut y = 1970u64;
    loop {
        let diy = if is_leap(y) { 366 } else { 365 };
        if days < diy {
            break;
        }
        days -= diy;
        y += 1;
    }
    let leap = is_leap(y);
    let month_days = [
        31u64,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0u64;
    for md in month_days.iter() {
        if days < *md {
            break;
        }
        days -= *md;
        m += 1;
    }
    (y, m + 1, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

#[tauri::command]
pub fn open_with(path: String, app: String) -> Result<(), String> {
    Command::new(&app)
        .arg(&path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_file(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    if p.exists() {
        return Err(format!("{} already exists", path));
    }
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::File::create(p).map_err(|e| e.to_string())?;
    Ok(path)
}

#[tauri::command]
pub fn create_dir(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    if p.exists() {
        return Err(format!("{} already exists", path));
    }
    std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    Ok(path)
}

#[tauri::command]
pub fn restore_items(paths: Vec<String>) -> Result<(), String> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })
        .ok_or_else(|| "no data home".to_string())?;
    let trash_files = data_home.join("Trash").join("files");
    let trash_info = data_home.join("Trash").join("info");

    for trashed in paths {
        let name = std::path::Path::new(&trashed)
            .file_name()
            .ok_or_else(|| format!("bad path {trashed}"))?
            .to_string_lossy()
            .into_owned();
        let info_path = trash_info.join(format!("{name}.trashinfo"));
        let info = std::fs::read_to_string(&info_path).map_err(|e| e.to_string())?;
        let original = info
            .lines()
            .find_map(|l| l.strip_prefix("Path="))
            .ok_or_else(|| "missing Path in trashinfo".to_string())?
            .to_string();
        let src = trash_files.join(&name);
        let dest = std::path::Path::new(&original);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::rename(&src, dest).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&info_path);
    }
    Ok(())
}

#[tauri::command]
pub fn copy_items(paths: Vec<String>, dest_dir: String) -> Result<Vec<String>, String> {
    let dest = std::path::Path::new(&dest_dir);
    if !dest.is_dir() {
        return Err(format!("{} is not a directory", dest_dir));
    }
    let mut out = Vec::new();
    for path in paths {
        let src = std::path::Path::new(&path);
        let name = src.file_name().ok_or_else(|| format!("bad path {path}"))?;
        let stem = src
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let ext = src
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let mut target = dest.join(name);
        if target.exists() {
            let mut i = 1;
            loop {
                let candidate = dest.join(format!("{stem}-{i}{ext}"));
                if !candidate.exists() {
                    target = candidate;
                    break;
                }
                i += 1;
            }
        }
        if src.is_dir() {
            copy_dir_recursive(src, &target)?;
        } else {
            std::fs::copy(src, &target).map_err(|e| e.to_string())?;
        }
        out.push(target.to_string_lossy().into_owned());
    }
    Ok(out)
}

#[tauri::command]
pub fn empty_trash() -> Result<(), String> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })
        .ok_or_else(|| "no data home".to_string())?;
    let trash_files = data_home.join("Trash").join("files");
    let trash_info = data_home.join("Trash").join("info");
    if trash_files.exists() {
        for entry in std::fs::read_dir(&trash_files).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let p = entry.path();
            if p.is_dir() {
                let _ = std::fs::remove_dir_all(&p);
            } else {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
    if trash_info.exists() {
        for entry in std::fs::read_dir(&trash_info).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let _ = std::fs::remove_file(entry.path());
        }
    }
    Ok(())
}
