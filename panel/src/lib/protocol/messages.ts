import type { EntryData, PageInfo, Value } from "$lib/types/model";

export type ClientMessage =
  | {
      type: "connect";
      token: string;
    }
  | {
      type: "create_entry";
      page_id: string;
      entry_id: string;
      data: EntryData;
    }
  | {
      type: "create_page";
      page: PageInfo;
    }
  | {
      type: "close_page";
      page_id: string;
    }
  | {
      type: "delete_entry";
      page_id: string;
      entry_id: string;
    }
  | {
      type: "delete_page";
      page_id: string;
    }
  | {
      type: "edit_entry";
      page_id: string;
      entry_id: string;
      field: string;
      value: Value;
    }
  | {
      type: "edit_page";
      page: PageInfo;
    }
  | {
      type: "open_page";
      page_id: string;
    }
  | {
      type: "publish";
    };

export type ServerMessage =
  | {
      type: "connection_result";
      result: ConnectionResult;
    }
  | {
      type: "entry_created";
      page_id: string;
      entry_id: string;
      data: EntryData;
    }
  | {
      type: "entry_deleted";
      page_id: string;
      entry_id: string;
    }
  | {
      type: "entry_edited";
      page_id: string;
      entry_id: string;
      field: string;
      value: Value;
    }
  | {
      type: "page_content";
      page_id: string;
      content: string;
    }
  | {
      type: "page_created";
      page: PageInfo;
    }
  | {
      type: "page_deleted";
      page_id: string;
    }
  | {
      type: "page_edited";
      page: PageInfo;
    }
  | {
      type: "page_tree";
      pages: PageInfo[];
    };

export type ConnectionResult =
  | "connected"
  | {
      error: {
        message: string;
      };
    };
