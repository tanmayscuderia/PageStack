# PageStack

PageStack is a Tauri desktop app for turning a set of images into a single PDF. It keeps the workflow simple: import images, reorder them, choose an output path, and generate the file.

## Features

- Drag and drop images into the app window
- Import a whole folder from the native dialog
- Reorder pages with drag and drop or the arrow buttons
- Choose compression presets for smaller or higher quality PDFs
- Generate a PDF with native Rust back-end processing
- Ship as a desktop app with a custom icon and installer support

## Tech Stack

- React
- Vite
- TypeScript
- Tauri v2
- Rust
- `printpdf`

## Run Locally

```bash
npm install
npm run tauri dev
```

If you only want the frontend in a browser:

```bash
npm run dev
```

## Build

```bash
npm run build
```

## Windows Installer

To build the Windows NSIS installer `.exe`:

```bash
npm run tauri build -- --runner cargo-xwin --target x86_64-pc-windows-msvc
```

The installer output is written to:

```bash
src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/
```

## Project Layout

```text
src/
  App.tsx
  main.tsx
  styles.css
  types.ts
src-tauri/
  Cargo.toml
  tauri.conf.json
  src/
    commands.rs
    lib.rs
    main.rs
    pipeline.rs
    presets.rs
    types.rs
```

## Notes

- The app is branded as `PageStack`.
- The current UI is optimized for desktop use.
- Windows installer assets live in `src-tauri/icons/`.

