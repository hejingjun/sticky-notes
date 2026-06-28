import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Note } from "../types/note";
import { newNote } from "../types/note";

const notes = ref<Note[]>([]);

function compareOrder(a: string, b: string): number {
  // Fixed-width 10-char hex: consistent lexicographic ordering
  return a < b ? -1 : a > b ? 1 : 0;
}

function sortNotes(a: Note, b: Note): number {
  if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
  return compareOrder(a.order, b.order);
}

function hexOrder(n: number): string {
  return n.toString(16).padStart(10, "0");
}

function midOrder(prev: string | null, next: string | null): string | null {
  const lo = prev !== null ? BigInt("0x" + prev) : 0n;
  const hi = next !== null ? BigInt("0x" + next) : 1n << 64n;
  const mid = (lo + hi) / 2n;
  // If mid doesn't fit in a unique 10-char hex between lo and hi, signal rebalance
  if (mid <= lo || mid >= hi) return null;
  return mid.toString(16).padStart(10, "0");
}

function needsRebalance(allNotes: Note[]): boolean {
  const tops = allNotes.filter((n) => !n.parent_id);
  return tops.some((n) => n.order.length !== 10 || /[^0-9a-f]/.test(n.order));
}

export function useNotes() {
  async function load() {
    const list = await invoke<Note[]>("list_notes");
    notes.value = [...list].sort(sortNotes);
  }

  async function add() {
    try {
      const tops = notes.value.filter((n) => !n.parent_id).sort(sortNotes);
      const last = tops.length > 0 ? tops[tops.length - 1].order : "0000000000";
      // If existing order is not hex (old data), generate fresh hex orders
      if (!/^[0-9a-f]{10,}$/i.test(last)) {
        for (let i = 0; i < tops.length; i++) {
          const newOrd = hexOrder(i);
          tops[i].order = newOrd;
          tops[i].updated_at = Date.now();
          await invoke("save_note", { note: tops[i] });
        }
        const note = newNote(hexOrder(tops.length));
        notes.value = [...notes.value, note];
        await invoke("save_note", { note });
      } else {
        const next = midOrder(last, null) ?? hexOrder(parseInt(last, 16) + 1);
        const note = newNote(next);
        notes.value = [...notes.value, note];
        await invoke("save_note", { note });
      }
    } catch (e) {
      console.error("add failed:", e);
    }
  }

  async function update(note: Note) {
    note.updated_at = Date.now();
    await invoke("save_note", { note });
    const idx = notes.value.findIndex((n) => n.id === note.id);
    if (idx >= 0) {
      const copy = [...notes.value];
      copy[idx] = { ...note };
      notes.value = copy.sort(sortNotes);
    }
  }

  async function remove(id: string) {
    await invoke("delete_note", { id });
    notes.value = notes.value.filter((n) => n.id !== id);
  }

  async function addSubtask(parentId: string) {
    const children = notes.value.filter((n) => n.parent_id === parentId);
    const lastOrder = children.length > 0 ? children[children.length - 1].order : hexOrder(0);
    const next = midOrder(lastOrder, null) ?? hexOrder(parseInt(lastOrder.slice(-10), 16) + 1);
    const note = newNote(next);
    note.parent_id = parentId;
    notes.value = [...notes.value, note];
    await invoke("save_note", { note });
  }

  async function toggleComplete(note: Note) {
    const newCompleted = !note.completed;
    const now = Date.now();
    note.completed = newCompleted;
    note.completed_at = newCompleted ? now : null;
    await update(note);
    // Cascade to subtasks only when completing (not when un-completing)
    if (newCompleted && !note.parent_id) {
      const children = notes.value.filter((n) => n.parent_id === note.id);
      for (const child of children) {
        if (!child.completed) {
          child.completed = true;
          child.completed_at = now;
          await update(child);
        }
      }
    }
  }

  async function togglePin(note: Note) {
    note.pinned = !note.pinned;
    await update(note);
  }

  /** Drag-drop reorder: place `id` right before `beforeId` (null = end). */
  async function reorder(id: string, beforeId: string | null) {
    const tops = notes.value
      .filter((n) => !n.parent_id && n.id !== id)
      .sort(sortNotes);

    // If any top-level note uses old format, migrate all
    if (needsRebalance(notes.value)) {
      tops.sort((a, b) => compareOrder(a.order, b.order));
      const insertAt = beforeId ? tops.findIndex((n) => n.id === beforeId) : tops.length;
      if (insertAt < 0) return;
      tops.splice(insertAt < 0 ? tops.length : insertAt, 0, notes.value.find((n) => n.id === id)!);
      for (let i = 0; i < tops.length; i++) {
        const newOrd = hexOrder(i);
        if (tops[i].order !== newOrd) {
          tops[i].order = newOrd;
          tops[i].updated_at = Date.now();
          await invoke("save_note", { note: tops[i] });
        }
      }
      notes.value = [...notes.value].sort(sortNotes);
      return;
    }

    // Try fractional midpoint
    const insertAt = beforeId ? tops.findIndex((n) => n.id === beforeId) : tops.length;
    if (insertAt < 0) return;
    const prev = insertAt > 0 ? tops[insertAt - 1].order : null;
    const next = insertAt < tops.length ? tops[insertAt].order : null;
    const newOrd = midOrder(prev, next);

    if (newOrd === null) {
      // Adjacent — rebalance all
      tops.splice(insertAt, 0, notes.value.find((n) => n.id === id)!);
      for (let i = 0; i < tops.length; i++) {
        const newOrd = hexOrder(i);
        if (tops[i].order !== newOrd) {
          tops[i].order = newOrd;
          tops[i].updated_at = Date.now();
          await invoke("save_note", { note: tops[i] });
        }
      }
      notes.value = [...notes.value].sort(sortNotes);
    } else {
      const note = notes.value.find((n) => n.id === id);
      if (!note) return;
      note.order = newOrd;
      await update(note);
    }
  }

  return { notes, load, add, addSubtask, update, remove, toggleComplete, togglePin, reorder };
}
