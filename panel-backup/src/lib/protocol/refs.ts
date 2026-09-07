import type { Value } from "$lib/types/model";

export function mapRefs(value: Value, fn: (ref: string) => string): Value {
  if (value === null || typeof value !== "object") return value;

  if ("reference" in value) return { reference: fn(value.reference) };

  if ("list" in value) {
    return { list: value.list.map((item) => mapRefs(item, fn)) };
  }

  if ("struct" in value) {
    return {
      struct: Object.fromEntries(
        Object.entries(value.struct).map(([key, child]) => [
          key,
          mapRefs(child, fn),
        ]),
      ),
    };
  }

  return value;
}

export function removeRefs(
  value: Value,
  matches: (ref: string) => boolean,
): { value: Value; changed: boolean } {
  if (value === null || typeof value !== "object") {
    return { value, changed: false };
  }

  if ("reference" in value) {
    if (!matches(value.reference)) return { value, changed: false };
    return { value: { reference: "" }, changed: true };
  }

  if ("list" in value) {
    let changed = false;
    const list: Value[] = [];

    for (const item of value.list) {
      if (
        item !== null &&
        typeof item === "object" &&
        "reference" in item &&
        matches(item.reference)
      ) {
        changed = true;
        continue;
      }

      const result = removeRefs(item, matches);
      if (result.changed) changed = true;
      list.push(result.value);
    }

    return changed
      ? { value: { list }, changed: true }
      : { value, changed: false };
  }

  if ("struct" in value) {
    let changed = false;
    const struct: Record<string, Value> = {};

    for (const [key, child] of Object.entries(value.struct)) {
      const result = removeRefs(child, matches);
      if (result.changed) changed = true;
      struct[key] = result.value;
    }

    return changed
      ? { value: { struct }, changed: true }
      : { value, changed: false };
  }

  return { value, changed: false };
}
