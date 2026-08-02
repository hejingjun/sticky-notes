import { ref, shallowRef } from "vue";
import type { Note } from "../types/note";

const MAX_HISTORY = 50;

// Deep copy notes array (structuredClone-like)
function cloneNotes(notes: Note[]): Note[] {
  return notes.map((n) => ({ ...n }));
}

const undoStack = ref<Note[][]>([]);
const redoStack = ref<Note[][]>([]);
const canUndo = shallowRef(false);
const canRedo = shallowRef(false);

function updateFlags() {
  canUndo.value = undoStack.value.length > 0;
  canRedo.value = redoStack.value.length > 0;
}

export function useUndoRedo() {
  /**
   * Save current state before a mutation.
   * Call this BEFORE modifying the notes array.
   */
  function saveSnapshot(currentNotes: Note[]) {
    undoStack.value.push(cloneNotes(currentNotes));
    if (undoStack.value.length > MAX_HISTORY) {
      undoStack.value.shift();
    }
    // New action invalidates redo history
    redoStack.value = [];
    updateFlags();
  }

  /**
   * Undo the last action. Returns the restored notes array, or null if nothing to undo.
   */
  function undo(currentNotes: Note[]): Note[] | null {
    if (undoStack.value.length === 0) return null;
    const previous = undoStack.value.pop()!;
    redoStack.value.push(cloneNotes(currentNotes));
    updateFlags();
    return previous;
  }

  /**
   * Redo the last undone action. Returns the restored notes array, or null if nothing to redo.
   */
  function redo(currentNotes: Note[]): Note[] | null {
    if (redoStack.value.length === 0) return null;
    const next = redoStack.value.pop()!;
    undoStack.value.push(cloneNotes(currentNotes));
    updateFlags();
    return next;
  }

  /**
   * Clear all history (e.g. after a full reload from backend).
   */
  function clearHistory() {
    undoStack.value = [];
    redoStack.value = [];
    updateFlags();
  }

  return { saveSnapshot, undo, redo, clearHistory, canUndo, canRedo };
}
