# AGENT.md - src/

## Overview
The primary frontend application written in Svelte 5 and TypeScript. This directory contains all components, views, and state management logic for the user interface. It is responsible purely for presentation and local state manipulation; it never accesses the filesystem directly.

## Components & Responsibility
*   **`src/App.svelte`:** The root component managing global state and orchestrating child views.
*   **`src/components/*`:** Highly modular components (e.g., `QuickLook.svelte`). Agents should work here to improve UI elements or user feedback loops.
*   **State Management:** All data fetching is handled via an API service that interfaces with the Tauri backend, ensuring data flow remains unidirectional from Rust -> JS.

## Agentic Focus Areas
*   Implementing complex UI state transitions (e.g., switching between Treemap and List view).
*   Improving accessibility or optimizing rendering performance for large datasets.
*   Creating new user-facing widgets that rely on the existing `api.ts` backend contracts.
