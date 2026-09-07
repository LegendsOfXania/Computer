import { describe, expect, it } from "vitest";
import { entryKey, parseEntryKey, summarizeEntry, type Entry } from "$lib/domain/model";

describe("EntryKey", () => {
  it("round-trips page and entry ids", () => {
    const key = entryKey("page", "entry");
    expect(key).toBe("page:entry");
    expect(parseEntryKey(key)).toEqual({ pageId: "page", entryId: "entry" });
  });

  it("rejects malformed keys", () => {
    expect(parseEntryKey("missing-separator")).toBeNull();
    expect(parseEntryKey(":entry")).toBeNull();
    expect(parseEntryKey("page:")).toBeNull();
  });
});

describe("summarizeEntry", () => {
  it("keeps search metadata independent from the detailed cache", () => {
    const entry: Entry = { id: "entry", entry_type: "quest", fields: { name: { text: "The Quest" } } };
    expect(summarizeEntry("page", entry, { entry_type: "quest", tags: ["story"], fields: [] })).toEqual({
      key: "page:entry", entryType: "quest", name: "The Quest", tags: ["story"],
    });
  });
});
