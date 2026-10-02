import { logError } from "$lib/utils/logger";
import { getCurrentWebview } from "@tauri-apps/api/webview";

interface FileDropHandlers {
  /** Lowercase, with the dot: ".mgx". */
  extension: string;
  onOver: () => void;
  onLeave: () => void;
  onDrop: (path: string) => void;
  onReject: () => void;
}

/** Subscribes to native file drops on the webview; returns the unlisten. */
export function listenFileDrop(handlers: FileDropHandlers): () => void {
  let cancelled = false;
  let unlisten: (() => void) | undefined;

  getCurrentWebview()
    .onDragDropEvent((event) => {
      const payload = event.payload;
      if (payload.type === "enter" || payload.type === "over") {
        handlers.onOver();
        return;
      }
      handlers.onLeave();
      if (payload.type !== "drop" || payload.paths.length === 0) return;
      const path = payload.paths[0];
      if (path.toLowerCase().endsWith(handlers.extension)) handlers.onDrop(path);
      else handlers.onReject();
    })
    .then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    })
    .catch((error) => logError("Failed to listen for file drops", error));

  return () => {
    cancelled = true;
    unlisten?.();
  };
}
