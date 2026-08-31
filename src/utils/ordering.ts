/**
 * Pure ordering utilities for fractional indexing.
 * No Vue, no Tauri — fully testable in isolation.
 */
import type { Note } from "../types/note";

export function compareOrder(a: string, b: string): number {
  // Fixed-width 10-char hex: consistent lexicographic ordering
  return a < b ? -1 : a > b ? 1 : 0;
}

export function sortNotes(a: Note, b: Note): number {
  if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
  return compareOrder(a.order, b.order);
}

export function hexOrder(n: number): string {
  return n.toString(16).padStart(10, "0");
}

export function midOrder(prev: string | null, next: string | null): string | null {
  const lo = prev !== null ? BigInt("0x" + prev) : 0n;
  const hi = next !== null ? BigInt("0x" + next) : 1n << 64n;
  const mid = (lo + hi) / 2n;
  // If mid doesn't fit in a unique 10-char hex between lo and hi, signal rebalance
  if (mid <= lo || mid >= hi) return null;
  return mid.toString(16).padStart(10, "0");
}

export function needsRebalance(allNotes: Note[]): boolean {
  const tops = allNotes.filter((n) => !n.parent_id);
  return tops.some((n) => n.order.length !== 10 || /[^0-9a-f]/.test(n.order));
}

/**
 * Compute the next order value for a new note appended after the last top-level note.
 * Returns the new hex order string.
 */
export function nextOrder(topNotes: Note[]): string {
  const sorted = [...topNotes].sort(sortNotes);
  const last = sorted.length > 0 ? sorted[sorted.length - 1].order : "0000000000";
  if (!/^[0-9a-f]{10,}$/i.test(last)) {
    // Old format — caller should rebalance first
    return hexOrder(sorted.length);
  }
  return midOrder(last, null) ?? hexOrder(parseInt(last, 16) + 1);
}

/**
 * Rebalance top-level notes: assign sequential hex orders.
 * Returns only the notes whose order actually changed (for efficient IPC).
 */
export function rebalanceNotes(topNotes: Note[]): Note[] {
  // [DRAG-FIX-7] 不排序：调用方（reorder）已通过 splice 将元素放到正确位置
  // 如果这里再 sort(sortNotes)，会按旧 order 排序，撤销 splice 的效果
  const changed: Note[] = [];
  for (let i = 0; i < topNotes.length; i++) {
    const newOrd = hexOrder(i);
    if (topNotes[i].order !== newOrd) {
      topNotes[i].order = newOrd;
      topNotes[i].updated_at = Date.now();
      changed.push(topNotes[i]);
    }
  }
  return changed;
}
