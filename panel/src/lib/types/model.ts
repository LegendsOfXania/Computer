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

export interface Entry extends EntryData {
  id: string;
}

export function getField(
  fields: Record<string, Value>,
  key: string,
): Value | undefined {
  return Object.hasOwn(fields, key) ? fields[key] : undefined;
}

export type PageType = "sequence" | "static";

export interface PageInfo {
  id: string;
  name: string;
  page_type: PageType;
  priority: number;
}

export interface PageContent {
  page: PageInfo;
  entries: Entry[];
}

export function parseEntryReference(
  currentPageId: string,
  value: string,
): { pageId: string; entryId: string } {
  const separatorIndex = value.indexOf(":");
  if (separatorIndex === -1) {
    return { pageId: currentPageId, entryId: value };
  }

  return {
    pageId: value.slice(0, separatorIndex),
    entryId: value.slice(separatorIndex + 1),
  };
}
