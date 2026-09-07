import { describe, expect, it } from "vitest";
import { mapRefs, removeRefs } from "$lib/domain/refs";

describe("references", () => {
  it("maps nested references", () => {
    const value = {
      struct: {
        a: { reference: "page:entry" },
        b: {
          list: [{ reference: "page:entry" }],
        },
      },
    };

    expect(mapRefs(value, (ref) => ref.replace("page:", "other:"))).toEqual({
      struct: {
        a: { reference: "other:entry" },
        b: {
          list: [{ reference: "other:entry" }],
        },
      },
    });
  });

  it("removes nested references without mutating unrelated branches", () => {
    const value = {
      list: [{ reference: "page:entry" }, { text: "keep" }],
    };

    expect(removeRefs(value, (ref) => ref === "page:entry")).toEqual({
      value: {
        list: [{ reference: "" }, { text: "keep" }],
      },
      changed: true,
    });
  });
});
