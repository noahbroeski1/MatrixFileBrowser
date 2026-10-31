// SPDX-License-Identifier: GPL-3.0-or-later
//! Core filesystem model shared by the Tauri shell and every view.

pub mod entry;
pub mod listing;
pub mod places;
pub mod vfs;

pub use entry::{stable_id, Entry, EntryKind, SortKey};
pub use listing::{list_dir, DirPage, ListOptions};
pub use places::{places, Place, PlaceKind};
pub use vfs::{LocalVfs, Vfs};
