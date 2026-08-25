export type NumberValue = { integer: number } | { float: number };

export type Value =
  | "null"
  | { number: NumberValue }
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

export type PageType = "sequence" | "static";

export interface PageInfo {
  id: string;
  name: string;
  page_type: PageType;
  priority: number;
}
