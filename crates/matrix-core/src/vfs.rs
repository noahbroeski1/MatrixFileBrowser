// SPDX-License-Identifier: GPL-3.0-or-later
//! Filesystem abstraction so archives, MTP devices and SFTP can plug in later
//! without touching the UI or the views.

use crate::entry::Entry;
use crate::listing::{list_dir, DirPage, ListOptions};
use std::io;
use std::path::Path;

pub trait Vfs: Send + Sync {
    fn list(&self, dir: &Path, opts: &ListOptions) -> io::Result<DirPage>;
    fn stat(&self, path: &Path) -> io::Result<Entry>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalVfs;

impl Vfs for LocalVfs {
    fn list(&self, dir: &Path, opts: &ListOptions) -> io::Result<DirPage> {
        list_dir(dir, opts)
    }

    fn stat(&self, path: &Path) -> io::Result<Entry> {
        Entry::from_path(path)
    }
}
