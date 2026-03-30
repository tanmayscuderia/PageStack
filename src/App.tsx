import { Fragment, useEffect, useMemo, useRef, useState, type DragEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { AppImage, GenerateResult, QualityPreset } from "./types";

export default function App() {
  const [images, setImages] = useState<AppImage[]>([]);
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [dropIndex, setDropIndex] = useState<number | null>(null);
  const [preset, setPreset] = useState<QualityPreset>("balanced");
  const [outputPath, setOutputPath] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<GenerateResult | null>(null);
  const [error, setError] = useState("");
  const dragIndexRef = useRef<number | null>(null);

  const totalInputBytes = useMemo(
    () => images.reduce((sum, img) => sum + (img.sizeBytes ?? 0), 0),
    [images]
  );
  const canGenerate = images.length > 0 && outputPath.trim().length > 0;
  const previews = useMemo(
    () =>
      images.map((img) => ({
        ...img,
        previewUrl: img.previewDataUrl ?? ""
      })),
    [images]
  );

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setup = async () => {
      const window = getCurrentWindow();
      unlisten = await window.onDragDropEvent(async (event) => {
        if (event.payload.type !== "drop" || !event.payload.paths.length) {
          return;
        }

        try {
          const dropped = await invoke<AppImage[]>("load_images_from_paths", {
            paths: event.payload.paths
          });
          if (dropped.length) {
            setImages((current) => mergeImages(current, dropped));
          }
        } catch (e) {
          setError(String(e));
        }
      });
    };

    void setup();

    return () => {
      unlisten?.();
    };
  }, []);

  async function pickFolder() {
    setError("");
    try {
      const folder = await open({
        directory: true,
        multiple: false,
        title: "Select an image folder"
      });

      if (typeof folder === "string" && folder.length > 0) {
        const files = await invoke<AppImage[]>("load_images_from_folder", {
          folderPath: folder
        });
        setImages((current) => mergeImages(current, files));
      }
    } catch (e) {
      setError(String(e));
    }
  }

  async function pickOutput() {
    setError("");
    try {
      const path = await save({
        title: "Choose output PDF",
        defaultPath: "output.pdf",
        filters: [{ name: "PDF", extensions: ["pdf"] }]
      });
      if (path) {
        setOutputPath(path);
      }
    } catch (e) {
      setError(String(e));
    }
  }

  async function generatePdf() {
    if (!canGenerate) {
      setError("Add images and an output path first.");
      return;
    }

    setLoading(true);
    setError("");
    setResult(null);

    try {
      const res = await invoke<GenerateResult>("generate_pdf", {
        request: {
          paths: images.map((i) => i.path),
          outputPath,
          preset
        }
      });
      setResult(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  function moveUp(index: number) {
    if (index === 0) return;
    const next = [...images];
    [next[index - 1], next[index]] = [next[index], next[index - 1]];
    setImages(next);
  }

  function moveDown(index: number) {
    if (index === images.length - 1) return;
    const next = [...images];
    [next[index + 1], next[index]] = [next[index], next[index + 1]];
    setImages(next);
  }

  function removeImage(index: number) {
    setImages((current) => current.filter((_, imageIndex) => imageIndex !== index));
  }

  function moveImage(fromIndex: number, insertionIndex: number) {
    setImages((current) => {
      const next = [...current];
      if (
        fromIndex < 0 ||
        fromIndex >= next.length ||
        insertionIndex < 0 ||
        insertionIndex > next.length
      ) {
        return current;
      }

      const [moved] = next.splice(fromIndex, 1);
      const nextIndex = fromIndex < insertionIndex ? insertionIndex - 1 : insertionIndex;
      next.splice(nextIndex, 0, moved);
      return next;
    });
  }

  function handleDragStart(event: DragEvent<HTMLElement>, index: number) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", String(index));
    setDragIndex(index);
    dragIndexRef.current = index;
  }

  function handleDragOver(event: DragEvent<HTMLElement>, index: number) {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
    const rect = event.currentTarget.getBoundingClientRect();
    const insertAfter = event.clientX > rect.left + rect.width / 2;
    const insertionIndex = index + (insertAfter ? 1 : 0);
    setDropIndex(insertionIndex);
  }

  function handleDragEnter(event: DragEvent<HTMLElement>, index: number) {
    handleDragOver(event, index);
  }

  function handleFilmstripDragOver(event: DragEvent<HTMLElement>) {
    if (dragIndexRef.current === null) {
      return;
    }

    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
    setDropIndex(images.length);
  }

  function clearDragState() {
    setDragIndex(null);
    setDropIndex(null);
    dragIndexRef.current = null;
  }

  function handleDrop() {
    const fromIndex = dragIndexRef.current;
    const insertionIndex = dropIndex;

    if (fromIndex === null || insertionIndex === null) {
      clearDragState();
      return;
    }

    if (insertionIndex !== fromIndex && insertionIndex !== fromIndex + 1) {
      moveImage(fromIndex, insertionIndex);
    }

    clearDragState();
  }

  return (
    <div className="appShell">
      <header className="topbar">
        <div className="topbarCopy">
          <h1>PageStack</h1>
          <p>Arrange images, choose an output path, and generate a compact PDF without extra clutter.</p>
        </div>
        <div className="statusRow" aria-label="Current document status">
          <div className="statusChip">
            <span className="statusLabel">Images</span>
            <span className="statusValue">{images.length}</span>
          </div>
          <div className="statusChip">
            <span className="statusLabel">Input size</span>
            <span className="statusValue">{formatBytes(totalInputBytes)}</span>
          </div>
          <div className="statusChip">
            <span className="statusLabel">Output</span>
            <span className="statusValue">{outputPath ? "Set" : "Missing"}</span>
          </div>
        </div>
      </header>

      <main className="workspace">
        <section className="panel importPanel">
          <div className="importTop">
            <div>
              <h2>Import</h2>
              <p>Drop files in the window, or use the controls to add a folder and choose the output file.</p>
            </div>
            <div className="sectionMeta">Output path</div>
          </div>

          <div className="importControls">
            <div className="dropzone">
              <strong>Drop images here</strong>
              <span>then reorder them in the queue below</span>
            </div>

            <div className="controlCluster">
              <button type="button" className="button secondary" onClick={pickFolder}>Pick folder</button>
              <button type="button" className="button secondary" onClick={pickOutput}>Output</button>
              <label className="field inlineField">
                <input
                  placeholder="C:\\docs\\output.pdf"
                  value={outputPath}
                  onChange={(e) => setOutputPath(e.target.value)}
                />
              </label>
              <button
                type="button"
                className="button primary"
                onClick={generatePdf}
                disabled={loading || !canGenerate}
              >
                {loading ? "Generating..." : "Generate PDF"}
              </button>
            </div>

            <label className="field presetField">
              <span>Compression preset</span>
              <select value={preset} onChange={(e) => setPreset(e.target.value as QualityPreset)}>
                <option value="small">Small</option>
                <option value="balanced">Balanced</option>
                <option value="high">High quality</option>
              </select>
            </label>
          </div>

          {loading && (
            <div className="loadingBar" aria-live="polite" aria-busy="true">
              <div className="loadingBarTrack">
                <div className="loadingBarFill" />
              </div>
              <div className="loadingStatus">
                <span className="spinner" />
                <span>Generating PDF, please wait...</span>
              </div>
            </div>
          )}

          {error && <div className="notice noticeError">{error}</div>}

          {result && (
            <div className="notice resultPanel">
              <div className="sectionHeading compact">
                <div>
                  <h2>Done</h2>
                  <p>The PDF was created successfully.</p>
                </div>
              </div>
              <div className="resultGrid">
                <div>
                  <span className="resultLabel">Output</span>
                  <span className="resultValue">{result.outputPath}</span>
                </div>
                <div>
                  <span className="resultLabel">Pages</span>
                  <span className="resultValue">{result.pageCount}</span>
                </div>
                <div>
                  <span className="resultLabel">Final size</span>
                  <span className="resultValue">{formatBytes(result.outputBytes)}</span>
                </div>
                <div>
                  <span className="resultLabel">Input size</span>
                  <span className="resultValue">{formatBytes(result.inputBytes)}</span>
                </div>
              </div>
            </div>
          )}
        </section>

        <section className="panel queuePanel">
          <div className="queueHeader">
            <div>
              <h2>Queue</h2>
              <p>Drag cards left and right to change page order.</p>
            </div>
            <div className="sectionMeta">{images.length ? "Ready for generation" : "No files loaded yet"}</div>
          </div>

          <div className="filmstrip" onDragOver={handleFilmstripDragOver} onDrop={handleDrop}>
            {images.length === 0 ? (
              <div className="emptyState">
                Add a folder or drop files here to build the document order.
              </div>
            ) : (
              <>
                {previews.map((img, index) => (
                  <Fragment key={img.path}>
                    {dropIndex === index && (
                      <div className="insertionMarker" aria-hidden="true">
                        <span className="insertionMarkerLine" />
                      </div>
                    )}
                    <article
                      className={`imageCard ${dragIndex === index ? "isDragging" : ""}`}
                      draggable
                      onDragStart={(event) => handleDragStart(event, index)}
                      onDragOver={(event) => handleDragOver(event, index)}
                      onDragEnter={(event) => handleDragEnter(event, index)}
                      onDrop={handleDrop}
                      onDragEnd={clearDragState}
                    >
                      <div className="thumb">
                        <img src={img.previewUrl} alt={img.name} loading="lazy" />
                      </div>
                      <button
                        type="button"
                        className="dragHandle"
                        draggable={false}
                        aria-label={`Drag ${img.name}`}
                      >
                        ↔ Drag
                      </button>
                      <div className="cardBody">
                        <div className="name">{img.name}</div>
                        <div className="path">{img.path}</div>
                        <div className="cardMeta">
                          <span>{index + 1}</span>
                          <span>{formatBytes(img.sizeBytes ?? 0)}</span>
                        </div>
                      </div>
                      <div className="cardActions">
                        <button
                          type="button"
                          className="iconButton"
                          onClick={() => moveUp(index)}
                          aria-label={`Move ${img.name} left`}
                        >
                          ←
                        </button>
                        <button
                          type="button"
                          className="iconButton"
                          onClick={() => moveDown(index)}
                          aria-label={`Move ${img.name} right`}
                        >
                          →
                        </button>
                        <button
                          type="button"
                          className="iconButton"
                          onClick={() => removeImage(index)}
                          aria-label={`Remove ${img.name}`}
                        >
                          ×
                        </button>
                      </div>
                    </article>
                  </Fragment>
                ))}
                {dropIndex === images.length && (
                  <div className="insertionMarker insertionMarkerEnd" aria-hidden="true">
                    <span className="insertionMarkerLine" />
                  </div>
                )}
              </>
            )}
          </div>
        </section>
      </main>
    </div>
  );
}

function formatBytes(bytes: number) {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(2)} ${units[unitIndex]}`;
}

function mergeImages(existing: AppImage[], incoming: AppImage[]) {
  const seen = new Set(existing.map((image) => image.path));
  const next = [...existing];

  for (const image of incoming) {
    if (seen.has(image.path)) {
      continue;
    }
    seen.add(image.path);
    next.push(image);
  }

  return next;
}
