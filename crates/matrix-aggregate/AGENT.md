# AGENT.md - crates/matrix-aggregate

## Overview
This crate is responsible for computationally heavy background processes related to directory analysis that do not fit into a simple file listing (e.g., calculating disk usage, finding total size of a folder). It provides parallel and cancellable methods for large-scale data aggregation.

## Key Responsibilities
*   **Parallelism:** Utilizes the `rayon` crate heavily to perform recursive directory traversals and calculations across multiple CPU cores simultaneously, maximizing speed.
*   **Rollup Calculation:** Computes cumulative metrics (total size, count of specific file types) for a given path efficiently.
*   **Cancellation/State Tracking:** Must expose mechanisms to allow the UI to monitor progress and potentially cancel long-running operations.

## Agentic Focus Areas
*   Implementing new aggregation metrics (e.g., finding the total number of unique authors or detecting common file extensions).
*   Optimizing the traversal algorithm for edge cases (like traversing across multiple mounted filesystems).
