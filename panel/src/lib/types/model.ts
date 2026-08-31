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

export interface ReferenceSchema {
  entry_type?: string;
  tags: string[];
}

export type Schema =
  | "null"
  | "integer"
  | "float"
  | "boolean"
  | "text"
  | { enumeration: string[] }
  | { reference: ReferenceSchema }
  | { struct: Field[] }
  | { list: Schema };

export interface Field {
  name: string;
  schema: Schema;
}

export interface EntryDefinition {
  entry_type: string;
  tags: string[];
  fields: Field[];
}

export type PageType = "sequence" | "static";

export const PAGE_TYPES: PageType[] = ["sequence", "static"];

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

export function defaultSchema(schema: Schema): Value {
  if (schema === "null") return "null";
  if (schema === "text") return { text: "" };
  if (schema === "integer") return { integer: 0 };
  if (schema === "float") return { float: 0 };
  if (schema === "boolean") return { boolean: false };
  if ("enumeration" in schema) return { enum: schema.enumeration[0] ?? "" };
  if ("reference" in schema) return { reference: "" };
  if ("list" in schema) return { list: [] };

  return {
    struct: Object.fromEntries(
      schema.struct.map((field) => [field.name, defaultSchema(field.schema)]),
    ),
  };
}
