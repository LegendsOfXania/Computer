export type TabKind = "file" | "home" | "settings";

export interface Tab {
  id: string;
  kind: TabKind;
  label: string;
  fileId?: string; // only when kind === 'file'
  dirty: boolean;
}

export type ConnectionStatus =
  | "disconnected"
  | "connecting"
  | "connected"
  | "stagging";

export interface FieldSchema {
  type: string;
  label: string;
  description?: string;
  required?: boolean;
  default?: unknown;
}

export interface EntryTypeDefinition {
  type: string;
  label: string;
  description?: string;
  fields: Record<string, FieldSchema>;
}

export interface Registry {
  entryTypes: EntryTypeDefinition[];
}
