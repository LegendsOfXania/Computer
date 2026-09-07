import type { Value } from "./model";

export function mapRefs(
  value: Value,
  map: (reference: string) => string,
): Value {
  if ("reference" in value) return { reference: map(value.reference) };
  if ("list" in value) return { list: value.list.map((item) => mapRefs(item, map)) };
  if ("struct" in value) {
    return {
      struct: Object.fromEntries(
        Object.entries(value.struct).map(([key, child]) => [key, mapRefs(child, map)]),
      ),
    };
  }
  return value;
}

export function removeRefs(
  value: Value,
  shouldRemove: (reference: string) => boolean,
): { value: Value; changed: boolean } {
  if ("reference" in value) {
    if (value.reference && shouldRemove(value.reference)) {
      return { value: { reference: "" }, changed: true };
    }
    return { value, changed: false };
  }

  if ("list" in value) {
    let changed = false;
    const list = value.list.map((item) => {
      const result = removeRefs(item, shouldRemove);
      changed ||= result.changed;
      return result.value;
    });
    return { value: changed ? { list } : value, changed };
  }

  if ("struct" in value) {
    let changed = false;
    const struct = Object.fromEntries(
      Object.entries(value.struct).map(([key, child]) => {
        const result = removeRefs(child, shouldRemove);
        changed ||= result.changed;
        return [key, result.value];
      }),
    );
    return { value: changed ? { struct } : value, changed };
  }

  return { value, changed: false };
}
