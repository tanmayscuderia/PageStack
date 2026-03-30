export type QualityPreset = "small" | "balanced" | "high";

export interface AppImage {
  path: string;
  name: string;
  sizeBytes?: number;
  previewDataUrl?: string;
}

export interface GenerateRequest {
  paths: string[];
  outputPath: string;
  preset: QualityPreset;
}

export interface GenerateResult {
  outputPath: string;
  outputBytes: number;
  inputBytes: number;
  pageCount: number;
}
