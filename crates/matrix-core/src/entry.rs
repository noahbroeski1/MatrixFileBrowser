// SPDX-License-Identifier: GPL-3.0-or-later
//! A single directory entry and its metadata.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Dir,
    File,
    Symlink,
    Other,
}

impl EntryKind {
    pub fn is_dir(self) -> bool {
        matches!(self, EntryKind::Dir)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Deterministic hex id derived from the absolute path, stable across calls.
    pub id: String,
    pub path: String,
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
    pub mtime_ms: i64,
    pub ctime_ms: i64,
    pub is_hidden: bool,
    pub mode: u32,
    pub ext: Option<String>,
    pub mime: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortKey {
    Name,
    Size,
    Modified,
    Type,
}

/// Stable 64-bit id for a path. Collisions are astronomically unlikely and
/// only affect selection identity, never filesystem access.
pub fn stable_id(path: &str) -> u64 {
    let hash = blake3::hash(path.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    u64::from_le_bytes(bytes)
}

fn ms_since_epoch(time: io::Result<std::time::SystemTime>) -> i64 {
    time.ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl Entry {
    /// Build an entry from a path without following symlinks.
    pub fn from_path(path: &Path) -> io::Result<Entry> {
        let meta = fs::symlink_metadata(path)?;
        let file_type = meta.file_type();

        let kind = if file_type.is_symlink() {
            EntryKind::Symlink
        } else if file_type.is_dir() {
            EntryKind::Dir
        } else if file_type.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());

        let path_str = path.to_string_lossy().into_owned();
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .filter(|e| !e.is_empty());

        let mime = mime_guess::from_path(path)
            .first_raw()
            .map(|m| m.to_string());

        // Symlink sizes describe the link target string, not the file; report 0
        // so treemaps do not attribute phantom bytes.
        let size = if kind == EntryKind::Symlink {
            0
        } else {
            meta.len()
        };

        Ok(Entry {
            id: format!("{:016x}", stable_id(&path_str)),
            path: path_str,
            name: name.clone(),
            kind,
            size,
            mtime_ms: ms_since_epoch(meta.modified()),
            ctime_ms: (meta.ctime() as i64) * 1000,
            is_hidden: name.starts_with('.'),
            mode: meta.mode(),
            ext,
            mime,
        })
    }

    pub fn is_dir(&self) -> bool {
        self.kind.is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_id_is_deterministic() {
        assert_eq!(stable_id("/tmp/a"), stable_id("/tmp/a"));
        assert_ne!(stable_id("/tmp/a"), stable_id("/tmp/b"));
    }

    #[test]
    fn reads_a_real_file() {
        let dir = std::env::temp_dir().join("matrix-core-test");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("hello.txt");
        fs::write(&file, b"hello").unwrap();
        let entry = Entry::from_path(&file).unwrap();
        assert_eq!(entry.kind, EntryKind::File);
        assert_eq!(entry.size, 5);
        assert_eq!(entry.ext.as_deref(), Some("txt"));
        assert!(!entry.is_hidden);
    }
}
