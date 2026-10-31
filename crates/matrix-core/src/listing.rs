// SPDX-License-Identifier: GPL-3.0-or-later
//! Paged, sorted directory listing.

use crate::entry::{Entry, SortKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirPage {
    pub path: String,
    pub entries: Vec<Entry>,
    pub next_offset: Option<u32>,
    pub total: u32,
}

#[derive(Debug, Clone)]
pub struct ListOptions {
    pub sort: SortKey,
    pub descending: bool,
    pub dirs_first: bool,
    pub show_hidden: bool,
    pub filter: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self {
            sort: SortKey::Name,
            descending: false,
            dirs_first: true,
            show_hidden: false,
            filter: None,
            offset: 0,
            limit: 500,
        }
    }
}

pub fn list_dir(dir: &Path, opts: &ListOptions) -> io::Result<DirPage> {
    let needle = opts
        .filter
        .as_ref()
        .map(|f| f.trim().to_lowercase())
        .filter(|f| !f.is_empty());

    let mut entries: Vec<Entry> = Vec::new();
    for item in fs::read_dir(dir)? {
        let Ok(item) = item else { continue };
        let Ok(entry) = Entry::from_path(&item.path()) else {
            continue;
        };
        if !opts.show_hidden && entry.is_hidden {
            continue;
        }
        if let Some(needle) = &needle {
            if !entry.name.to_lowercase().contains(needle) {
                continue;
            }
        }
        entries.push(entry);
    }

    sort_entries(&mut entries, opts.sort, opts.descending, opts.dirs_first);

    let total = entries.len() as u32;
    let start = (opts.offset as usize).min(entries.len());
    let end = (start + opts.limit.max(1) as usize).min(entries.len());
    let next_offset = (end < entries.len()).then_some(end as u32);

    Ok(DirPage {
        path: dir.to_string_lossy().into_owned(),
        entries: entries[start..end].to_vec(),
        next_offset,
        total,
    })
}

fn sort_entries(entries: &mut [Entry], key: SortKey, descending: bool, dirs_first: bool) {
    entries.sort_by(|a, b| {
        if dirs_first {
            let a_dir = a.is_dir();
            let b_dir = b.is_dir();
            if a_dir != b_dir {
                // Directories always lead, regardless of sort direction.
                return b_dir.cmp(&a_dir);
            }
        }

        let ordering = match key {
            SortKey::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortKey::Size => a.size.cmp(&b.size),
            SortKey::Modified => a.mtime_ms.cmp(&b.mtime_ms),
            SortKey::Type => a
                .ext
                .as_deref()
                .unwrap_or("")
                .cmp(b.ext.as_deref().unwrap_or(""))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        };

        if descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn lists_and_sorts() {
        let root = std::env::temp_dir().join("matrix-core-listing");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("beta")).unwrap();
        fs::create_dir_all(root.join("alpha")).unwrap();
        File::create(root.join("zeta.txt")).unwrap();
        File::create(root.join("alpha.txt")).unwrap();

        let page = list_dir(&root, &ListOptions::default()).unwrap();
        let names: Vec<_> = page.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "beta", "alpha.txt", "zeta.txt"]);

        let page = list_dir(
            &root,
            &ListOptions {
                offset: 2,
                limit: 2,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(page.entries.len(), 2);
        assert_eq!(page.next_offset, None);
        assert_eq!(page.total, 4);
    }
}
