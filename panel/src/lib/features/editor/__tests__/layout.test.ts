import { describe, expect, it } from "vitest";
import { layoutSequenceEntries } from "$lib/features/editor/layout";

describe("sequence layout", () => {
  it("lays out local references from left to right", () => {
    const result = layoutSequenceEntries("p", [
      { id: "a", entry_type: "node", fields: { next: { reference: "p:b" } } },
      { id: "b", entry_type: "node", fields: {} },
    ]);
    expect(result.edges).toHaveLength(1);
    expect(result.nodes.find((node) => node.id === "a")?.position.x).toBeLessThan(
      result.nodes.find((node) => node.id === "b")?.position.x ?? Infinity,
    );
  });
});
