import type { EntryData, PageInfo, Value } from "$lib/types/model";

export type ClientMessage =
  | {
      type: "connect";
      token: string;
    }
  | {
      type: "create_entry";
      entry_key: string;
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
      entry_key: string;
    }
  | {
      type: "delete_page";
      page_id: string;
    }
  | {
      type: "edit_entry";
      entry_key: string;
      field: string;
      value: Value;
    }
  | {
      type: "edit_page";
      page: PageInfo;
    }
  | {
      type: "get_entry_data";
      entry_key: string;
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
      type: "library";
      pages: PageInfo[];
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
      type: "entry_data";
      entry_key: string;
      data: EntryData;
    }
  | {
      type: "entry_created";
      entry_key: string;
      data: EntryData;
    }
  | {
      type: "entry_deleted";
      entry_key: string;
    }
  | {
      type: "entry_edited";
      entry_key: string;
      field: string;
      value: Value;
    };

export type ConnectionResult =
  | "connected"
  | {
      error: {
        message: string;
      };
    };
