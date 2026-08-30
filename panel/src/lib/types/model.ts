export type Value =
  | "null"
  | { integer: number }
  | { float: number }
  | { boolean: boolean }
  | { text: string }
  | { enum: string }
  | { reference: string }
  | { struct: Record<string, Value> }
  | { list: Value[] };

export interface EntryData {
  entry_type: string;
  fields: Record<string, Value>;
}

export interface Entry extends EntryData {
  id: string;
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

  return typeof value === "object" && "text" in value ? value.text : entry.id;
}

export function defaultValue(value: Value): Value {
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
      Object.entries(value.struct).map(([key, child]) => [
        key,
        defaultValue(child),
      ]),
    ),
  };
}
