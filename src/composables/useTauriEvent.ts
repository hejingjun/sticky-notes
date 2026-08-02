import { onUnmounted } from "vue";
import { listen, type EventCallback } from "@tauri-apps/api/event";

/**
 * Auto-cleanup Tauri event listener.
 * Registers on setup, unregisters on component unmount.
 *
 * Usage:
 *   useTauriEvent<boolean>("penetrate-changed", (e) => { penetrating.value = e.payload });
 */
export function useTauriEvent<T>(event: string, handler: EventCallback<T>) {
  let unlisten: (() => void) | null = null;

  listen<T>(event, handler).then((fn) => {
    unlisten = fn;
  });

  onUnmounted(() => {
    unlisten?.();
  });
}
