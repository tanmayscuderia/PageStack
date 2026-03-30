import { useCallback, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppImage } from "../types";

export function useImageQueue() {
  const [images, setImages] = useState<AppImage[]>([]);

  const totalInputBytes = useMemo(
    () => images.reduce((sum, img) => sum + (img.sizeBytes ?? 0), 0),
    [images]
  );

  const loadImagesFromPaths = useCallback(async (paths: string[]) => {
    if (!paths.length) {
      return;
    }

    const dropped = await invoke<AppImage[]>("load_images_from_paths", { paths });
    if (dropped.length) {
      setImages((current) => mergeImages(current, dropped));
    }
  }, []);

  const loadImagesFromFolder = useCallback(async (folderPath: string) => {
    const files = await invoke<AppImage[]>("load_images_from_folder", {
      folderPath
    });
    if (files.length) {
      setImages((current) => mergeImages(current, files));
    }
  }, []);

  const moveImage = useCallback((fromIndex: number, insertionIndex: number) => {
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
  }, []);

  const moveUp = useCallback((index: number) => {
    if (index === 0) return;
    setImages((current) => {
      const next = [...current];
      [next[index - 1], next[index]] = [next[index], next[index - 1]];
      return next;
    });
  }, []);

  const moveDown = useCallback((index: number) => {
    setImages((current) => {
      if (index >= current.length - 1) {
        return current;
      }

      const next = [...current];
      [next[index + 1], next[index]] = [next[index], next[index + 1]];
      return next;
    });
  }, []);

  const removeImage = useCallback((index: number) => {
    setImages((current) => current.filter((_, imageIndex) => imageIndex !== index));
  }, []);

  return {
    images,
    totalInputBytes,
    loadImagesFromFolder,
    loadImagesFromPaths,
    moveDown,
    moveImage,
    moveUp,
    removeImage,
    setImages
  };
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
