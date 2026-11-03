# AGENT.md - crates/matrix-thumbs

## Overview
This dedicated crate handles all aspects of the visual thumbnail pipeline. Its primary function is to take a path and generate various derivative images (thumbnails) efficiently, supporting multiple formats and sizes required by different views (List, Treemap, Mosaic).

## Key Responsibilities
*   **Generation:** Runs external tooling (like `ffmpeg`, `pdftoppm`) or internal logic to create visual representations of files.
*   **Caching:** Implements a robust, content-addressed disk cache to ensure that thumbnails are generated only once and retrieved quickly on subsequent sessions, preventing excessive CPU load and I/O operations.
*   **Error Handling:** Must gracefully fail when necessary external tools (e.g., `ffmpeg`) or file types are missing, falling back to basic icons instead of crashing the application.

## Agentic Focus Areas
*   Adding support for new file type formats that require specific thumbnail generation logic.
*   Optimizing cache lookup and invalidation policies.
*   Improving the parallel execution of image processing tasks (using `rayon`).
