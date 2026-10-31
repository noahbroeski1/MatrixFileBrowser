// SPDX-License-Identifier: GPL-3.0-or-later
//! Sidebar places: home, XDG directories, mounted volumes and trash.

use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaceKind {
    Home,
    Xdg,
    Mount,
    Trash,
    Root,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub id: String,
    pub label: String,
    pub path: String,
    pub kind: PlaceKind,
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
}

fn push(
    places: &mut Vec<Place>,
    seen: &mut HashSet<String>,
    label: &str,
    path: PathBuf,
    kind: PlaceKind,
) {
    if !path.is_dir() {
        return;
    }
    let key = path.to_string_lossy().into_owned();
    if !seen.insert(key.clone()) {
        return;
    }
    places.push(Place {
        id: key.clone(),
        label: label.to_string(),
        path: key,
        kind,
    });
}

pub fn places() -> Vec<Place> {
    let mut places = Vec::new();
    let mut seen = HashSet::new();

    if let Some(home) = home_dir() {
        push(
            &mut places,
            &mut seen,
            "Home",
            home.clone(),
            PlaceKind::Home,
        );

        for (label, dir) in [
            ("Desktop", "Desktop"),
            ("Documents", "Documents"),
            ("Downloads", "Downloads"),
            ("Music", "Music"),
            ("Pictures", "Pictures"),
            ("Videos", "Videos"),
            ("Projects", "Projects"),
        ] {
            push(
                &mut places,
                &mut seen,
                label,
                home.join(dir),
                PlaceKind::Xdg,
            );
        }
    }

    let trash = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|h| h.join(".local/share")))
        .map(|d| d.join("Trash/files"));
    if let Some(trash) = trash {
        push(&mut places, &mut seen, "Trash", trash, PlaceKind::Trash);
    }

    push(
        &mut places,
        &mut seen,
        "Filesystem",
        PathBuf::from("/"),
        PlaceKind::Root,
    );

    for mount in mounts() {
        push(&mut places, &mut seen, &mount.1, mount.0, PlaceKind::Mount);
    }

    places
}

/// Parse /proc/mounts and keep real, user-relevant filesystems.
fn mounts() -> Vec<(PathBuf, String)> {
    const ALLOWED: &[&str] = &[
        "ext2", "ext3", "ext4", "xfs", "btrfs", "zfs", "f2fs", "vfat", "exfat", "ntfs", "ntfs3",
        "fuseblk", "iso9660", "hfsplus", "apfs",
    ];
    const SKIP_PREFIX: &[&str] = &["/proc", "/sys", "/dev", "/run", "/snap", "/var/lib"];

    let Ok(contents) = fs::read_to_string("/proc/mounts") else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for line in contents.lines() {
        let mut fields = line.split_whitespace();
        let device = fields.next().unwrap_or_default();
        let mount_point = fields.next().unwrap_or_default();
        let fstype = fields.next().unwrap_or_default();

        if !ALLOWED.contains(&fstype) {
            continue;
        }
        if SKIP_PREFIX.iter().any(|p| mount_point.starts_with(p)) {
            continue;
        }
        let path = Path::new(mount_point);
        if !path.is_dir() {
            continue;
        }

        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| device.trim_start_matches("/dev/").to_string());

        out.push((path.to_path_buf(), label));
    }
    out
}
