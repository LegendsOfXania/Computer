import { describe, expect, it } from "vitest";
import { mergeSearchQueries, parseSearchQuery } from "$lib/features/search/query";

describe("search query", () => {
  it("parses aliases, flags, values and text", () => {
    const query = parseSearchQuery("foo !e !t:Story !tags:a,b");
    expect(query.flags).toEqual(new Set(["entry"]));
    expect(query.values.get("tag")).toEqual(["story"]);
    expect(query.values.get("tags")).toEqual(["a", "b"]);
    expect(query.text).toBe("foo");
  });

  it("merges queries without sharing mutable collections", () => {
    const merged = mergeSearchQueries(parseSearchQuery("!entry"), parseSearchQuery("hello"));
    expect(merged.flags).toEqual(new Set(["entry"]));
    expect(merged.text).toBe("hello");
  });
});
