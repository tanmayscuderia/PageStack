# Project Memory

## What we built

This repository now contains PageStack, a Tauri desktop app for turning ordered images into a PDF.

### Frontend
- React + Vite app in `src/`
- Main UI in `src/App.tsx`
- Shared types in `src/types.ts`
- Styling in `src/styles.css`
- Bootstrap entry in `src/main.tsx`

### Backend
- Tauri v2 config in `src-tauri/tauri.conf.json`
- Rust crate config in `src-tauri/Cargo.toml`
- Commands in `src-tauri/src/commands.rs`
- PDF generation pipeline in `src-tauri/src/pipeline.rs`
- Preset settings in `src-tauri/src/presets.rs`
- Shared Rust types in `src-tauri/src/types.rs`
- Tauri entrypoints in `src-tauri/src/lib.rs` and `src-tauri/src/main.rs`

### Repository setup
- Root `package.json`
- Root `index.html`
- Root `tsconfig.json`
- Root `vite.config.ts`
- Root `.gitignore`
- `PROJECT_MEMORY.md` and `Status_Update.md` for living project notes
- Root `README.md` for public-facing project docs

## Current behavior
- Drop image files into the app window
- Pick a folder of images from the native dialog
- Pick an output PDF path from the native dialog
- Reorder images with drag-and-drop and up/down controls
- Choose a compression preset
- Generate a PDF from the selected images
- Show thumbnails and queue ordering in a wrapped card layout

## Interaction details
- Drag-and-drop now uses the Tauri webview drag-drop event instead of HTML5 drop objects
- Folder and save dialogs now use the frontend `@tauri-apps/plugin-dialog` API
- The app capability file grants `core:default` and `dialog:default` permissions to the main window
- Rust IPC structs serialize as camelCase so the frontend and backend payloads line up naturally
- Tauri bundling now has Windows icon assets at `src-tauri/icons/icon.ico` and `src-tauri/icons/icon.png`
- Windows NSIS builds use `cargo-xwin` with `x86_64-pc-windows-msvc`

## Important notes
- Frontend build has already been verified with `npm run build`
- Node dependencies have already been installed with `npm install`
- Rust compilation has been verified with `cargo check`
- The current Rust/PDF implementation is designed for Tauri v2 and `printpdf` 0.9
- Windows installers are emitted under `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/` as `*-setup.exe`

## Useful files
- [`src/App.tsx`](./src/App.tsx)
- [`src-tauri/src/pipeline.rs`](./src-tauri/src/pipeline.rs)
- [`src-tauri/src/commands.rs`](./src-tauri/src/commands.rs)
- [`src-tauri/tauri.conf.json`](./src-tauri/tauri.conf.json)
