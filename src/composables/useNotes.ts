import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Note } from "../types/note";
import { newNote } from "../types/note";
import { sortNotes, hexOrder, midOrder, needsRebalance, nextOrder, rebalanceNotes } from "../utils/ordering";
import { useUndoRedo } from "./useUndoRedo";

const notes = ref<Note[]>([]);
const { saveSnapshot, undo: _undo, redo: _redo, clearHistory, canUndo, canRedo } = useUndoRedo();

export function useNotes() {
  async function load() {
    const list = await invoke<Note[]>("list_notes");
    notes.value = [...list].sort(sortNotes);
    clearHistory();
  }

  async function saveNotes(notesToSave: Note[]) {
    for (const n of notesToSave) {
      await invoke("save_note", { note: n });
    }
  }

  /** Persist restored notes to backend and update local state */
  async function persistRestored(restored: Note[]) {
    // Save all restored notes to backend
    await saveNotes(restored);
    // Delete notes that exist in backend but not in restored
    const restoredIds = new Set(restored.map((n) => n.id));
    for (const n of notes.value) {
      if (!restoredIds.has(n.id)) {
        await invoke("delete_note", { id: n.id });
      }
    }
    notes.value = [...restored].sort(sortNotes);
  }

  async function add() {
    try {
      saveSnapshot(notes.value);
      const tops = notes.value.filter((n) => !n.parent_id);
      if (needsRebalance(notes.value)) {
        const changed = rebalanceNotes(tops);
        await saveNotes(changed);
      }
      const ord = nextOrder(notes.value.filter((n) => !n.parent_id));
      const note = newNote(ord);
      notes.value = [...notes.value, note];
      await invoke("save_note", { note });
    } catch (e) {
      console.error("add failed:", e);
    }
  }

  async function update(note: Note) {
    saveSnapshot(notes.value);
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
    saveSnapshot(notes.value);
    await invoke("delete_note", { id });
    notes.value = notes.value.filter((n) => n.id !== id);
  }

  async function addSubtask(parentId: string) {
    saveSnapshot(notes.value);
    const children = notes.value.filter((n) => n.parent_id === parentId);
    const lastOrder = children.length > 0 ? children[children.length - 1].order : hexOrder(0);
    const next = midOrder(lastOrder, null) ?? hexOrder(parseInt(lastOrder.slice(-10), 16) + 1);
    const note = newNote(next);
    note.parent_id = parentId;
    notes.value = [...notes.value, note];
    await invoke("save_note", { note });
  }

  async function toggleComplete(note: Note) {
    saveSnapshot(notes.value);
    const newCompleted = !note.completed;
    const now = Date.now();
    note.completed = newCompleted;
    note.completed_at = newCompleted ? now : null;
    await invoke("save_note", { note });
    const idx = notes.value.findIndex((n) => n.id === note.id);
    if (idx >= 0) {
      const copy = [...notes.value];
      copy[idx] = { ...note };
      notes.value = copy.sort(sortNotes);
    }
    if (newCompleted && !note.parent_id) {
      const children = notes.value.filter((n) => n.parent_id === note.id);
      for (const child of children) {
        if (!child.completed) {
          child.completed = true;
          child.completed_at = now;
          await invoke("save_note", { note: child });
          const ci = notes.value.findIndex((n) => n.id === child.id);
          if (ci >= 0) notes.value[ci] = { ...child };
        }
      }
      notes.value = [...notes.value].sort(sortNotes);
    }
  }

  async function togglePin(note: Note) {
    saveSnapshot(notes.value);
    note.pinned = !note.pinned;
    await invoke("save_note", { note });
    const idx = notes.value.findIndex((n) => n.id === note.id);
    if (idx >= 0) {
      const copy = [...notes.value];
      copy[idx] = { ...note };
      notes.value = copy.sort(sortNotes);
    }
  }

  async function reorder(id: string, beforeId: string | null) {
    saveSnapshot(notes.value);
    const tops = notes.value
      .filter((n) => !n.parent_id && n.id !== id)
      .sort(sortNotes);

    if (needsRebalance(notes.value)) {
      const insertAt = beforeId ? tops.findIndex((n) => n.id === beforeId) : tops.length;
      if (insertAt < 0) return;
      tops.splice(insertAt, 0, notes.value.find((n) => n.id === id)!);
      const changed = rebalanceNotes(tops);
      await saveNotes(changed);
      notes.value = [...notes.value].sort(sortNotes);
      return;
    }

    const insertAt = beforeId ? tops.findIndex((n) => n.id === beforeId) : tops.length;
    if (insertAt < 0) return;
    const prev = insertAt > 0 ? tops[insertAt - 1].order : null;
    const next = insertAt < tops.length ? tops[insertAt].order : null;
    const newOrd = midOrder(prev, next);

    if (newOrd === null) {
      tops.splice(insertAt, 0, notes.value.find((n) => n.id === id)!);
      const changed = rebalanceNotes(tops);
      await saveNotes(changed);
      notes.value = [...notes.value].sort(sortNotes);
    } else {
      const note = notes.value.find((n) => n.id === id);
      if (!note) return;
      note.order = newOrd;
      await invoke("save_note", { note });
      const idx = notes.value.findIndex((n) => n.id === id);
      if (idx >= 0) {
        const copy = [...notes.value];
        copy[idx] = { ...note };
        notes.value = copy.sort(sortNotes);
      }
    }
  }

  async function undo() {
    const restored = _undo(notes.value);
    if (restored) await persistRestored(restored);
  }

  async function redo() {
    const restored = _redo(notes.value);
    if (restored) await persistRestored(restored);
  }

  return { notes, load, add, addSubtask, update, remove, toggleComplete, togglePin, reorder, undo, redo, canUndo, canRedo };
}
