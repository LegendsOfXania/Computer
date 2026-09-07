import type {
  Entry,
  EntryData,
  EntryDefinition,
  EntryKey,
  EntrySummary,
  PageInfo,
  Value,
} from "$lib/domain/model";

export type ClientMessage =
  | { type: "connect"; token: string }
  | { type: "create_entry"; entry_key: EntryKey; data: EntryData }
  | { type: "create_page"; page: PageInfo }
  | { type: "close_page"; page_id: string }
  | { type: "delete_entry"; entry_key: EntryKey }
  | { type: "delete_page"; page_id: string }
  | { type: "edit_entry"; entry_key: EntryKey; field: string; value: Value }
  | { type: "edit_page"; page: PageInfo }
  | { type: "get_entry_data"; entry_key: EntryKey }
  | { type: "open_page"; page_id: string }
  | { type: "move_entry"; entry_key: EntryKey; target_page_id: string }
  | { type: "replace_entry"; entry_key: EntryKey; entry_type: string }
  | { type: "publish" };

export type ServerMessage =
  | { type: "connection_result"; result: ConnectionResult }
  | {
      type: "library";
      pages: PageInfo[];
      entry_definitions: EntryDefinition[];
      entry_summaries: EntrySummary[];
    }
  | { type: "page_content"; page: PageInfo; entries: Entry[] }
  | { type: "page_created"; page: PageInfo }
  | { type: "page_deleted"; page_id: string }
  | { type: "page_edited"; page: PageInfo }
  | { type: "entry_data"; entry_key: EntryKey; data: EntryData }
  | { type: "entry_error"; entry_key: EntryKey; message: string }
  | { type: "entry_created"; entry_key: EntryKey; data: EntryData }
  | { type: "entry_deleted"; entry_key: EntryKey }
  | { type: "entry_replaced"; entry_key: EntryKey; data: EntryData }
  | {
      type: "entry_edited";
      entry_key: EntryKey;
      field: string;
      value: Value;
    };

export type ConnectionResult = "connected" | { error: { message: string } };
