export function formatAppError(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  const structured = parseStructuredError(message);

  if (structured) {
    switch (structured.code) {
      case "NO_SUPPORTED_IMAGES":
        return "No supported images were found in that folder.";
      case "TOO_MANY_IMAGES":
        return "This batch is too large. Try fewer images at once.";
      case "FILE_TOO_LARGE":
        return "One image exceeds the 50 MB limit.";
      case "UNSUPPORTED_IMAGE":
        return "One selected file is not a supported image format.";
      case "WRITE_PDF":
        return "Could not save the PDF. Check the output path.";
      case "OPEN_IMAGE":
      case "DECODE_IMAGE":
        return "One image could not be read. It may be corrupted.";
      default:
        return structured.message ?? message;
    }
  }

  if (message.includes("no supported images were found")) {
    return "No supported images were found in that folder.";
  }

  if (message.includes("too many images selected")) {
    return "This batch is too large. Try fewer images at once.";
  }

  if (message.includes("file is too large")) {
    return "One image exceeds the 50 MB limit.";
  }

  if (message.includes("unsupported image format")) {
    return "One selected file is not a supported image format.";
  }

  if (message.includes("failed to write output pdf")) {
    return "Could not save the PDF. Check the output path and try again.";
  }

  return message;
}

type StructuredError = {
  code?: string;
  message?: string;
};

function parseStructuredError(message: string): StructuredError | null {
  try {
    const parsed = JSON.parse(message) as StructuredError;
    if (parsed && typeof parsed === "object") {
      return parsed;
    }
    return null;
  } catch {
    return null;
  }
}
