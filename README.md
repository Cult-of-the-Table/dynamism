# Dynamism

Dynamism is a Rust desktop application that searches the web for a query, extracts and semantically segments the returned pages, embeds them locally, projects the embeddings into 2D space with UMAP, and lets you explore the resulting semantic clusters in an interactive scatter plot.

## How It Works

Each search runs through an async pipeline:

1. **Web Search (`dynamism::websearch`)** — Queries DuckDuckGo for the entered search term.
2. **Download & Extraction (`dynamism::reqwest`, `dynamism::scraper`)** — Concurrently fetches result pages, sanitizes the HTML, and extracts readable article text with `readability-rust` (falling back to plain DOM text extraction).
3. **Semantic Segmentation & Embedding (`dynamism::segmentation`)** — Splits each page into sentences via `icu_segmenter`, embeds sentences locally with `fastembed` (`nomic-embed-text-v1.5`), and merges adjacent sentences whose cosine similarity exceeds the threshold (`1.0 - sigma`).
4. **2D Projection (`dynamism::umap`)** — Reduces the chunk embeddings to 2D coordinates using GPU-accelerated `fast-umap` (Burn `wgpu` backend, cosine metric).
5. **Local Storage (`dynamism::db`)** — Writes the projected coordinates, source URL, and chunk snippet to a local LanceDB database at `./db/` (`embeds` table), clearing previous search data on each new query.
6. **Interactive Plot (`frontend`)** — Reads the stored chunks from LanceDB and renders them as an interactive 2D scatter plot.

## Frontend

The `frontend` crate is a dark-themed desktop UI built with `iced` (Tokio runtime) and `iced_plot` following Iced's `Message` / `update` / `view` architecture:

- **Interactive Scatter Plot (`iced_plot::PlotWidget`)** — Displays the 2D UMAP coordinates as a scatter series with automatic scaling on updates, smooth pan/zoom, and GPU-backed rendering.
- **Point Picking & Detail Panel** — Clicking any point highlights it (enlarged red marker) and slides open a scrollable side panel showing the chunk's **URL**, **Snippet**, and **Text**. Clicking away clears the selection and collapses the panel.
- **Floating Search Bar** — A query input field and **search** button are overlaid in the bottom-left corner of the plot via `stack!`. Submitting a query clears the input and runs the full `load` → `read` pipeline asynchronously inside an `iced::Task` so the window stays responsive.
- **Clean State Reset Between Searches** — When new results arrive (`Message::DB`), the previous plot series and active point selection are cleared before rendering the new embeddings.

## Getting Started

Run the desktop app from the workspace root:

```sh
cargo run --release
```

Enter a search query in the input box at the bottom-left and click **search**. Terminal progress bars (`indicatif`) track embedding and UMAP epochs while the search runs. Once complete, the points appear on the plot and can be panned, zoomed, and clicked to inspect their source URL and snippet.

> **Note:** On the first run, `fastembed` automatically downloads `nomic-embed-text-v1.5` into `.fastembed_cache/`. Each new search overwrites the local `./db/` LanceDB table.

## Tech Stack

- **UI & Plotting:** `iced`, `iced_plot`
- **Embeddings & UMAP:** `fastembed` (`nomic-embed-text-v1.5`), `fast-umap`, `burn` (`wgpu`)
- **Vector Storage:** `lancedb`, `arrow-array`, `arrow-schema`, `serde_arrow`
- **Search & Scraping:** `websearch`, `reqwest`, `readability-rust`, `sanitize_html`, `scraper`, `icu_segmenter`
- **Async & Telemetry:** `tokio`, `futures`, `indicatif`
