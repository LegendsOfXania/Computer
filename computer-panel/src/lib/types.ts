export type PageType = "sequence" | "static" | "manifest";
export type Value =
  | string
  | number
  | boolean
  | null
  | Value[]
  | { [key: string]: Value };
export interface EntryData {
  entryType: string;
  fields: Record<string, Value>;
}
export interface Entry {
  id: string;
  data: EntryData;
  position: { x: number; y: number };
}
export interface PageConnection {
  id: string;
  source: string;
  target: string;
}
export interface Page {
  id: string;
  name: string;
  pageType: PageType;
  priority: number;
  entries: Entry[];
  connections: PageConnection[];
}
export type ConnectionState = "connecting" | "connected" | "disconnected";
export type Selection =
  | { type: "page"; pageId: string }
  | { type: "entry"; pageId: string; entryId: string }
  | null;
