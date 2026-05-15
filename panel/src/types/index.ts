export type FileType = "sequence" | "manifest" | "scene" | "static";

export interface ComputerFile {
  id: string;
  name: string;
  type: FileType;
  priority: number;
  version: string;
  entries: FileEntry[];
}

export interface FileEntry {
  id: string;
  name: string;
  type: string;
  fields: Record<string, unknown>;
}

export interface Folder {
  id: string;
  name: string;
  children: FolderChild[];
}

export type FolderChild =
  | { kind: "folder"; folder: Folder }
  | { kind: "file"; file: ComputerFile };

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
