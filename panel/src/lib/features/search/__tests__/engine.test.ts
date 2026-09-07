import { describe, expect, it } from "vitest";
import { search } from "$lib/features/search/engine";
import { parseSearchQuery } from "$lib/features/search/query";

describe("search engine", () => {
  const pages = [{ id: "p", name: "Main", page_type: "sequence" as const, priority: 0 }];
  const definitions = { quest: { entry_type: "quest", tags: ["story"], fields: [] } };
  const index = [{ key: "p:e", entryType: "quest", name: "Find the key", tags: ["story"] }];

  it("searches summaries without requiring an Entry cache", () => {
    const result = search(parseSearchQuery("key"), index, pages, definitions);
    expect(result.entries).toHaveLength(1);
  });
});
