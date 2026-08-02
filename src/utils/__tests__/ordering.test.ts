import { describe, it, expect } from "vitest";
import { compareOrder, hexOrder, midOrder, needsRebalance, sortNotes } from "../ordering";
import type { Note } from "../../types/note";

// Helper to create a minimal Note for testing ordering
function fakeNote(id: string, order: string, pinned = false, parent_id: string | null = null): Note {
  return {
    id,
    title: "",
    parent_id,
    order,
    completed: false,
    pinned,
    color: "#333333",
    created_at: 0,
    updated_at: 0,
    deleted_at: null,
    conflict_id: null,
    due_date: null,
    remind_at: null,
    completed_at: null,
  } as Note;
}

describe("hexOrder", () => {
  it("zero-pads to 10 characters", () => {
    expect(hexOrder(0)).toBe("0000000000");
    expect(hexOrder(1)).toBe("0000000001");
    expect(hexOrder(255)).toBe("00000000ff");
    expect(hexOrder(65535)).toBe("00000fffff");
  });

  it("handles large numbers", () => {
    const max32 = 0xffffffff;
    expect(hexOrder(max32)).toBe("ffffffff");
    // padStart(10, "0") ensures 10-char width
    expect(hexOrder(max32).length).toBe(10);
  });

  it("produces lexicographically sortable strings", () => {
    const orders = [10, 2, 100, 1, 0].map(hexOrder);
    const sorted = [...orders].sort();
    expect(sorted).toEqual([hexOrder(0), hexOrder(1), hexOrder(2), hexOrder(10), hexOrder(100)]);
  });
});

describe("compareOrder", () => {
  it("returns -1 when a < b", () => {
    expect(compareOrder("0000000000", "0000000001")).toBe(-1);
  });

  it("returns 1 when a > b", () => {
    expect(compareOrder("0000000002", "0000000001")).toBe(1);
  });

  it("returns 0 when equal", () => {
    expect(compareOrder("0000000042", "0000000042")).toBe(0);
  });

  it("handles empty strings", () => {
    expect(compareOrder("", "0000000000")).toBe(-1);
    expect(compareOrder("0000000000", "")).toBe(1);
  });
});

describe("sortNotes", () => {
  it("pinned notes come first regardless of order", () => {
    const a = fakeNote("a", "0000000005", true);
    const b = fakeNote("b", "0000000001", false);
    expect(sortNotes(a, b)).toBe(-1);
    expect(sortNotes(b, a)).toBe(1);
  });

  it("non-pinned notes sorted by order", () => {
    const a = fakeNote("a", "0000000001", false);
    const b = fakeNote("b", "0000000005", false);
    expect(sortNotes(a, b)).toBe(-1);
  });

  it("pinned notes sorted by order among themselves", () => {
    const a = fakeNote("a", "0000000001", true);
    const b = fakeNote("b", "0000000005", true);
    expect(sortNotes(a, b)).toBe(-1);
  });

  it("full sort produces correct order", () => {
    const notes = [
      fakeNote("c", "0000000003", false),
      fakeNote("a", "0000000001", true),
      fakeNote("b", "0000000002", false),
      fakeNote("d", "0000000000", true),
    ];
    const sorted = [...notes].sort(sortNotes).map((n) => n.id);
    expect(sorted).toEqual(["d", "a", "c", "b"]); // pinned first (d<a by order), then non-pinned
  });
});

describe("midOrder", () => {
  it("returns midpoint between two orders", () => {
    const result = midOrder("0000000000", "0000000002");
    expect(result).toBe("0000000001");
  });

  it("returns midpoint when inserting at start (prev=null)", () => {
    const result = midOrder(null, "0000000004");
    // (0x0 + 0x4) / 2 = 0x2
    expect(result).toBe("0000000002");
  });

  it("returns midpoint when inserting at end (next=null)", () => {
    const result = midOrder("0000000004", null);
    // (0x4 + 2^64) / 2 — should be a huge number
    expect(result).not.toBeNull();
    expect(result!.length).toBe(10);
    // Should be greater than prev
    expect(result! > "0000000004").toBe(true);
  });

  it("returns null when no space between adjacent keys", () => {
    // 0x1 and 0x2 → mid = (1+2)/2 = 1 → mid <= lo
    const result = midOrder("0000000001", "0000000002");
    expect(result).toBeNull();
  });

  it("returns null when prev equals next", () => {
    const result = midOrder("0000000005", "0000000005");
    expect(result).toBeNull();
  });

  it("produces 10-char hex output", () => {
    const result = midOrder("0000000000", "0000ffff00");
    expect(result).toMatch(/^[0-9a-f]{10}$/);
  });

  it("result is strictly between lo and hi", () => {
    for (let i = 0; i < 100; i++) {
      const lo = hexOrder(i * 100);
      const hi = hexOrder(i * 100 + 200);
      const mid = midOrder(lo, hi);
      expect(mid).not.toBeNull();
      expect(mid! > lo).toBe(true);
      expect(mid! < hi).toBe(true);
    }
  });

  it("repeated midpoints converge (10 rounds)", () => {
    let lo: string | null = "0000000000";
    let hi: string | null = "000000000a";
    for (let i = 0; i < 10; i++) {
      const mid = midOrder(lo, hi);
      if (mid === null) break; // ran out of space
      expect(mid > lo!).toBe(true);
      expect(mid < hi!).toBe(true);
      lo = mid;
    }
  });
});

describe("needsRebalance", () => {
  it("returns false when all top-level orders are valid 10-char hex", () => {
    const notes = [
      fakeNote("a", "0000000000"),
      fakeNote("b", "0000000001"),
    ];
    expect(needsRebalance(notes)).toBe(false);
  });

  it("returns true when an order is not 10 chars", () => {
    const notes = [
      fakeNote("a", "0000000000"),
      fakeNote("b", "abc"), // too short
    ];
    expect(needsRebalance(notes)).toBe(true);
  });

  it("returns true when an order has non-hex chars", () => {
    const notes = [
      fakeNote("a", "0000000000"),
      fakeNote("b", "hello world"), // non-hex
    ];
    expect(needsRebalance(notes)).toBe(true);
  });

  it("ignores subtasks (notes with parent_id)", () => {
    const notes = [
      fakeNote("a", "0000000000"),
      fakeNote("b", "invalid", false, "a"), // subtask with bad order — ignored
    ];
    expect(needsRebalance(notes)).toBe(false);
  });

  it("returns false for empty list", () => {
    expect(needsRebalance([])).toBe(false);
  });
});
