// SPDX-License-Identifier: GPL-3.0-or-later
//! Parallel directory aggregation.
//!
//! `tree_stats` builds a depth-limited tree whose nodes carry rolled-up byte
//! counts, file counts and the newest mtime found beneath them. Nodes deeper
//! than `max_depth` are summed without allocating per-entry structs, which is
//! what keeps a 500k-file home directory cheap.

use matrix_core::{Entry, EntryKind};
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeStats {
    pub node: Entry,
    pub children: Vec<TreeStats>,
    pub total_bytes: u64,
    pub file_count: u64,
    pub newest_ms: i64,
    /// True when children were folded away because of the depth limit.
    pub truncated: bool,
}

/// Build a tree for `root`, stopping expansion at `max_depth` (root is depth 0).
/// `cancelled` is polled throughout; a cancelled run returns `Err("cancelled")`.
pub fn tree_stats<F>(root: &Path, max_depth: u32, cancelled: F) -> Result<TreeStats, String>
where
    F: Fn() -> bool + Sync,
{
    let entry = Entry::from_path(root).map_err(|e| e.to_string())?;
    if entry.kind != EntryKind::Dir {
        return Ok(TreeStats {
            total_bytes: entry.size,
            file_count: 1,
            newest_ms: entry.mtime_ms,
            node: entry,
            children: Vec::new(),
            truncated: false,
        });
    }

    let mut stats =
        build(root, entry, 0, max_depth, &cancelled).ok_or_else(|| "cancelled".to_string())?;
    sort_by_size(&mut stats.children);
    Ok(stats)
}

fn sort_by_size(children: &mut [TreeStats]) {
    children.sort_by_key(|a| std::cmp::Reverse(a.total_bytes));
    for child in children.iter_mut() {
        sort_by_size(&mut child.children);
    }
}

fn read_children(dir: &Path) -> Vec<Entry> {
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };
    read.filter_map(|item| item.ok())
        .filter_map(|item| Entry::from_path(&item.path()).ok())
        .collect()
}

fn build<F>(
    dir: &Path,
    entry: Entry,
    depth: u32,
    max_depth: u32,
    cancelled: &F,
) -> Option<TreeStats>
where
    F: Fn() -> bool + Sync,
{
    if cancelled() {
        return None;
    }

    let mut stats = TreeStats {
        total_bytes: if entry.kind == EntryKind::Dir {
            0
        } else {
            entry.size
        },
        file_count: if entry.kind == EntryKind::Dir { 0 } else { 1 },
        newest_ms: entry.mtime_ms,
        node: entry,
        children: Vec::new(),
        truncated: false,
    };

    if stats.node.kind != EntryKind::Dir {
        return Some(stats);
    }

    let children = read_children(dir);

    if depth >= max_depth {
        // Fold everything below this node into its totals without allocating
        // a TreeStats per descendant.
        let rolled: Vec<(u64, u64, i64)> = children
            .par_iter()
            .map(|child| rollup(Path::new(&child.path), child, cancelled))
            .collect();
        for (bytes, count, newest) in rolled {
            stats.total_bytes += bytes;
            stats.file_count += count;
            stats.newest_ms = stats.newest_ms.max(newest);
        }
        stats.truncated = !children.is_empty();
        return Some(stats);
    }

    let child_stats: Vec<TreeStats> = children
        .par_iter()
        .filter_map(|child| {
            build(
                Path::new(&child.path),
                child.clone(),
                depth + 1,
                max_depth,
                cancelled,
            )
        })
        .collect();

    if cancelled() {
        return None;
    }

    for child in &child_stats {
        stats.total_bytes += child.total_bytes;
        stats.file_count += child.file_count;
        stats.newest_ms = stats.newest_ms.max(child.newest_ms);
    }
    stats.children = child_stats;

    Some(stats)
}

fn rollup<F>(path: &Path, entry: &Entry, cancelled: &F) -> (u64, u64, i64)
where
    F: Fn() -> bool + Sync,
{
    if cancelled() {
        return (0, 0, entry.mtime_ms);
    }
    if entry.kind != EntryKind::Dir {
        return (entry.size, 1, entry.mtime_ms);
    }

    let children = read_children(path);
    let partial: Vec<(u64, u64, i64)> = children
        .par_iter()
        .map(|child| rollup(Path::new(&child.path), child, cancelled))
        .collect();

    let mut bytes = 0u64;
    let mut count = 0u64;
    let mut newest = entry.mtime_ms;
    for (b, c, n) in partial {
        bytes += b;
        count += c;
        newest = newest.max(n);
    }
    (bytes, count, newest)
}

/// Convenience wrapper that never cancels, for tests and one-shot CLI use.
pub fn tree_stats_blocking(root: &Path, max_depth: u32) -> Result<TreeStats, String> {
    let never = AtomicBool::new(false);
    tree_stats(root, max_depth, || never.load(Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn rolls_up_sizes() {
        let root = std::env::temp_dir().join("matrix-aggregate-test");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("a/b")).unwrap();
        File::create(root.join("top.bin"))
            .unwrap()
            .write_all(&[0u8; 100])
            .unwrap();
        File::create(root.join("a/mid.bin"))
            .unwrap()
            .write_all(&[0u8; 200])
            .unwrap();
        File::create(root.join("a/b/deep.bin"))
            .unwrap()
            .write_all(&[0u8; 300])
            .unwrap();

        let stats = tree_stats_blocking(&root, 4).unwrap();
        assert_eq!(stats.total_bytes, 600);
        assert_eq!(stats.file_count, 3);

        // Depth 1 keeps only the first level; "a" still reports its full size.
        let shallow = tree_stats_blocking(&root, 1).unwrap();
        assert_eq!(shallow.total_bytes, 600);
        let a = shallow
            .children
            .iter()
            .find(|c| c.node.name == "a")
            .unwrap();
        assert_eq!(a.total_bytes, 500);
        assert!(a.children.is_empty());
        assert!(a.truncated);
    }

    #[test]
    fn cancellation_stops_the_walk() {
        let root = std::env::temp_dir();
        let stats = tree_stats(&root, 8, || true);
        assert!(stats.is_err());
    }
}
