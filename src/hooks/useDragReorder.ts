import { useCallback, useState, type DragEvent } from "react";
import type { AppImage } from "../types";

type UseDragReorderArgs = {
  items: AppImage[];
  moveImage: (fromIndex: number, insertionIndex: number) => void;
};

export function useDragReorder({ items, moveImage }: UseDragReorderArgs) {
  const [dragPath, setDragPath] = useState<string | null>(null);
  const [dropIndex, setDropIndex] = useState<number | null>(null);

  const clearDragState = useCallback(() => {
    setDragPath(null);
    setDropIndex(null);
  }, []);

  const handleDragStart = useCallback((event: DragEvent<HTMLElement>, imagePath: string) => {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", imagePath);
    setDragPath(imagePath);
  }, []);

  const handleDragOver = useCallback((event: DragEvent<HTMLElement>, index: number) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
    const rect = event.currentTarget.getBoundingClientRect();
    const insertAfter = event.clientX > rect.left + rect.width / 2;
    setDropIndex(index + (insertAfter ? 1 : 0));
  }, []);

  const handleDragEnter = useCallback(
    (event: DragEvent<HTMLElement>, index: number) => {
      handleDragOver(event, index);
    },
    [handleDragOver]
  );

  const handleFilmstripDragOver = useCallback(
    (event: DragEvent<HTMLElement>) => {
      if (dragPath === null) {
        return;
      }

      event.preventDefault();
      event.dataTransfer.dropEffect = "move";
      setDropIndex(items.length);
    },
    [dragPath, items.length]
  );

  const handleDrop = useCallback(
    (event: DragEvent<HTMLElement>) => {
      event.preventDefault();
      event.stopPropagation();

      const sourcePath = event.dataTransfer.getData("text/plain") || dragPath;
      const insertionIndex = dropIndex;
      const fromIndex = sourcePath ? items.findIndex((image) => image.path === sourcePath) : -1;

      if (fromIndex === -1 || insertionIndex === null) {
        clearDragState();
        return;
      }

      if (insertionIndex !== fromIndex && insertionIndex !== fromIndex + 1) {
        moveImage(fromIndex, insertionIndex);
      }

      clearDragState();
    },
    [clearDragState, dragPath, dropIndex, items, moveImage]
  );

  return {
    dragPath,
    dropIndex,
    clearDragState,
    handleDragEnter,
    handleDragOver,
    handleDragStart,
    handleDrop,
    handleFilmstripDragOver
  };
}
