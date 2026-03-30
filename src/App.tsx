import { Fragment, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useImageQueue } from "./hooks/useImageQueue";
import { useDragReorder } from "./hooks/useDragReorder";
import { ImageCard } from "./components/ImageCard";
import { formatAppError } from "./lib/errors";
import { invoke } from "@tauri-apps/api/core";
import type { GenerateResult, QualityPreset } from "./types";

export default function App() {
  const [preset, setPreset] = useState<QualityPreset>("balanced");
  const [outputPath, setOutputPath] = useState("");
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const [result, setResult] = useState<GenerateResult | null>(null);
  const [error, setError] = useState("");
  const { images, totalInputBytes, loadImagesFromFolder, loadImagesFromPaths, moveDown, moveImage, moveUp, removeImage } =
    useImageQueue();

  const canGenerate = images.length > 0 && outputPath.trim().length > 0;
  const {
    dragPath,
    dropIndex,
    clearDragState,
    handleDragEnter,
    handleDragOver,
    handleDragStart,
    handleDrop,
    handleFilmstripDragOver
  } = useDragReorder({ items: images, moveImage });

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setup = async () => {
      const window = getCurrentWindow();
      unlisten = await window.onDragDropEvent(async (event) => {
        if (event.payload.type !== "drop" || !event.payload.paths.length) {
          return;
        }

        try {
          await loadImagesFromPaths(event.payload.paths);
        } catch (e) {
          setError(formatAppError(e));
        }
      });
    };

    void setup();

    return () => {
      unlisten?.();
    };
  }, [loadImagesFromPaths]);

  useEffect(() => {
    let unlistenProgress: (() => void) | null = null;

    if (!loading) {
      setProgress(null);
      return undefined;
    }

    const setup = async () => {
      unlistenProgress = await listen<[number, number]>("pdf_progress", (event) => {
        setProgress({ done: event.payload[0], total: event.payload[1] });
      });
    };

    void setup();

    return () => {
      unlistenProgress?.();
    };
  }, [loading]);

  async function pickFolder() {
    setError("");
    try {
      const folder = await open({
        directory: true,
        multiple: false,
        title: "Select an image folder"
      });

      if (typeof folder === "string" && folder.length > 0) {
        await loadImagesFromFolder(folder);
      }
    } catch (e) {
      setError(formatAppError(e));
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
      setError(formatAppError(e));
    }
  }

  async function generatePdf() {
    if (!canGenerate) {
      setError("Add images and an output path first.");
      return;
    }

    setLoading(true);
    setProgress(null);
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
      setError(formatAppError(e));
    } finally {
      setLoading(false);
    }
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
              <button type="button" className="button secondary" onClick={pickFolder}>
                Pick folder
              </button>
              <button type="button" className="button secondary" onClick={pickOutput}>
                Output
              </button>
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
                <div
                  className="loadingBarFill"
                  style={{
                    width: progress ? `${Math.max(4, (progress.done / progress.total) * 100)}%` : "45%",
                    animation: progress ? "none" : undefined
                  }}
                />
              </div>
              <div className="loadingStatus">
                <span className="spinner" />
                <span>
                  {progress ? `${progress.done} / ${progress.total} images processed` : "Generating PDF, please wait..."}
                </span>
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
                {images.map((img, index) => (
                  <Fragment key={img.path}>
                    {dropIndex === index && (
                      <div className="insertionMarker" aria-hidden="true">
                        <span className="insertionMarkerLine" />
                      </div>
                    )}
                    <ImageCard
                      image={img}
                      index={index}
                      isDragging={dragPath === img.path}
                      onDragStart={handleDragStart}
                      onDragOver={handleDragOver}
                      onDragEnter={handleDragEnter}
                      onDrop={handleDrop}
                      onDragEnd={clearDragState}
                      onMoveUp={moveUp}
                      onMoveDown={moveDown}
                      onRemove={removeImage}
                    />
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
