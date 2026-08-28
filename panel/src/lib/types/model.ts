export type Value =
  | "null"
  | { float: number }
  | { integer: number }
  | { boolean: boolean }
  | { text: string }
  | { enum: string }
  | { reference: string }
  | { struct: Record<string, Value> }
  | { list: Value[] };

export interface Entry {
  id: string;
  entry_type: string;
  fields: Record<string, Value>;
}
export type PageType = "sequence" | "static";
export interface PageInfo {
  id: string;
  name: string;
  page_type: PageType;
  priority: number;
}

export function displayName(entry: Entry): string {
  const value = entry.fields.name;
  return typeof value === "object" && value !== null && "text" in value
    ? value.text
    : entry.id;
}
export function parseEntryReference(currentPageId: string, value: string) {
  const i = value.indexOf(":");
  return i < 0
    ? { pageId: currentPageId, entryId: value }
    : { pageId: value.slice(0, i), entryId: value.slice(i + 1) };
}
export function blankLike(value: Value): Value {
  if (value === "null") return "null";
  if ("text" in value) return { text: "" };
  if ("enum" in value) return { enum: "" };
  if ("reference" in value) return { reference: "" };
  if ("integer" in value) return { integer: 0 };
  if ("float" in value) return { float: 0 };
  if ("boolean" in value) return { boolean: false };
  if ("list" in value) return { list: [] };
  return {
    struct: Object.fromEntries(
      Object.entries(value.struct).map(([k, v]) => [k, blankLike(v)]),
    ),
  };
}
