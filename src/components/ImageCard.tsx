import { memo, type DragEvent } from "react";
import type { AppImage } from "../types";

type ImageCardProps = {
  image: AppImage;
  index: number;
  isDragging: boolean;
  onDragStart: (event: DragEvent<HTMLElement>, imagePath: string) => void;
  onDragOver: (event: DragEvent<HTMLElement>, index: number) => void;
  onDragEnter: (event: DragEvent<HTMLElement>, index: number) => void;
  onDrop: (event: DragEvent<HTMLElement>) => void;
  onDragEnd: () => void;
  onMoveUp: (index: number) => void;
  onMoveDown: (index: number) => void;
  onRemove: (index: number) => void;
};

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

export const ImageCard = memo(function ImageCard({
  image,
  index,
  isDragging,
  onDragStart,
  onDragOver,
  onDragEnter,
  onDrop,
  onDragEnd,
  onMoveUp,
  onMoveDown,
  onRemove
}: ImageCardProps) {
  return (
    <article
      className={`imageCard ${isDragging ? "isDragging" : ""}`}
      draggable
      onDragStart={(event) => onDragStart(event, image.path)}
      onDragOver={(event) => onDragOver(event, index)}
      onDragEnter={(event) => onDragEnter(event, index)}
      onDrop={onDrop}
      onDragEnd={onDragEnd}
      aria-grabbed={isDragging}
    >
      <div className="thumb">
        <img src={toFileUrl(image.previewPath)} alt={image.name} loading="lazy" />
      </div>
      <button type="button" className="dragHandle" draggable={false} aria-label={`Drag ${image.name}`}>
        ↔ Drag
      </button>
      <div className="cardBody">
        <div className="name">{image.name}</div>
        <div className="path">{image.path}</div>
        <div className="cardMeta">
          <span>{index + 1}</span>
          <span>{formatBytes(image.sizeBytes ?? 0)}</span>
        </div>
      </div>
      <div className="cardActions">
        <button
          type="button"
          className="iconButton"
          onClick={() => onMoveUp(index)}
          aria-label={`Move ${image.name} left`}
        >
          ←
        </button>
        <button
          type="button"
          className="iconButton"
          onClick={() => onMoveDown(index)}
          aria-label={`Move ${image.name} right`}
        >
          →
        </button>
        <button
          type="button"
          className="iconButton"
          onClick={() => onRemove(index)}
          aria-label={`Remove ${image.name}`}
        >
          ×
        </button>
      </div>
    </article>
  );
});

function toFileUrl(path?: string) {
  if (!path) {
    return "";
  }

  const normalized = path.replace(/\\/g, "/");
  return normalized.startsWith("/") ? `file://${normalized}` : `file:///${normalized}`;
}
