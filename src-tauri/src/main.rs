// SPDX-License-Identifier: GPL-3.0-or-later
//! MatrixFileBrowser — a richly featured FOSS file browser for Linux.

mod commands;
mod state;
mod watcher;

use state::AppState;

fn main() {
    // WebKitGTK's DMA-BUF renderer is broken on many NVIDIA setups; disabling
    // it costs little and avoids blank or stuttering views.
    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/proc/driver/nvidia/version").exists() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::list_dir,
            commands::tree_stats,
            commands::thumbnails,
            commands::places,
            commands::reveal,
            commands::open_path,
            commands::preview_text,
            commands::home_dir,
            commands::rename_path,
            commands::move_items,
            commands::trash_items,
            watcher::watch_start,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start MatrixFileBrowser");
}
