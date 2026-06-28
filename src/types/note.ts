export interface Note {
  id: string;
  title: string;
  parent_id: string | null;
  order: string;
  completed: boolean;
  pinned: boolean;
  color: string;
  created_at: number;
  updated_at: number;
  deleted_at: number | null;
  conflict_id: string | null;
  due_date: number | null;
  remind_at: number | null;
  completed_at: number | null;
}

export const COLORS = [
  "#444444", // gray
  "#e06c75", // red
  "#d19a66", // orange
  "#e5c07b", // yellow
  "#98c379", // green
  "#56b6c2", // cyan
  "#61afef", // blue
  "#c678dd", // purple
] as const;

export const DEFAULT_COLOR = COLORS[0];

export function newNote(order: string, color = DEFAULT_COLOR): Note {
  const now = Date.now();
  return {
    id: crypto.randomUUID(),
    title: "",
    parent_id: null,
    order,
    completed: false,
    pinned: false,
    color,
    created_at: now,
    updated_at: now,
    deleted_at: null,
    conflict_id: null,
    due_date: null,
    remind_at: null,
    completed_at: null,
  };
}
