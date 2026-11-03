# AGENT.md - src-tauri

## Overview
This directory contains the Tauri shell code. It is the critical "glue" layer that bridges the gap between the JavaScript/TypeScript frontend and the Rust backend logic. It initializes the application state, registers command handlers, and manages the communication channel.

## Key Responsibilities
*   **Command Registration:** Exposes safe APIs to the JS side by calling specific functions in the Rust crates (`matrix-core`, etc.).
*   **State Management:** Manages global application state accessible both from JS and exposed to Rust commands (e.g., current user settings, theme).
*   **Initialization:** Ensures that core services, like filesystem watchers or database connections, are initialized correctly when the app starts up.

## Agentic Focus Areas
*   Adding new command handlers for features developed in other parts of the codebase.
*   Refactoring communication patterns (e.g., changing from request/response to event-driven updates).
*   Improving error handling and logging across the bridge layer.
