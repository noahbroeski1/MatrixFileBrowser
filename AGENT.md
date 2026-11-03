# AGENT.md

## Overview
This repository is MatrixFileBrowser, a rich, cross-platform, read-only file browser designed to make large directory structures highly navigable. The goal is always performance and stability by strictly separating the UI (Svelte/TypeScript) from the system interaction (Rust).

## Architecture Principle
The entire project operates on two core contracts:
1.  **System Contract:** Only Rust code within `crates/` may perform I/O or interact with the OS filesystem, ensuring safety and predictable behavior.
2.  **Communication Contract:** The UI communicates solely through the Tauri backend shell (`src-tauri`) which calls defined functions exposed by the core crates.

## Agentic Focus Areas
*   **UI Logic (Svelte):** Working on view state management, component rendering, or improving user interaction flow.
*   **API Layer (Tauri/JS):** Updating how data is requested from Rust or modifying client-side routing based on backend changes.
*   **Core Logic (Rust Crates):** Implementing new filesystem features, optimizing I/O routines, or refining the underlying data models (`Entry` model).

## Setup & Execution
Use `npm run dev` for a browser-only UI test (no system dependencies required) and `npm run tauri dev` for a full build including OS interactions.
