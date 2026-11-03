# AGENT.md - crates/matrix-core

## Overview
This crate is the foundation of the entire application. It defines core data models (like the `Entry` struct) and provides the primary filesystem abstraction layer (`Vfs trait`). Its purpose is to provide a generalized, paginated view of the directory structure regardless of the underlying OS or file system type (e.g., local disk vs. network mount).

## Key Responsibilities
*   **Data Modeling:** Defines canonical data structures used throughout the app for file/folder representations.
*   **Filesystem Abstraction:** The `Vfs` trait ensures that higher layers are agnostic to whether they are reading from a local drive, SFTP, or a mock backend.
*   **Pagination Logic:** Manages fetching directory contents in controlled batches (`read_dir` with limits), which is crucial for performance and preventing browser stalls on huge directories.

## Agentic Focus Areas
*   Refining the `Vfs` trait to support new storage backends (e.g., cloud API calls).
*   Improving the efficiency or completeness of metadata extraction from entries.
*   Implementing new filesystem behaviors that need a core data model change (e.g., better handling for symbolic links).
