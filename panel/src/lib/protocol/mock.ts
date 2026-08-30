import type { Entry, EntryData, PageInfo, Value } from "$lib/types/model";
import type { ClientMessage, ServerMessage } from "./messages";
import { entriesByPage, pages } from "./data";

type Handler = (message: ServerMessage) => void;

export interface MockConnection {
  send(message: ClientMessage): void;
  subscribe(handler: Handler): () => void;
  disconnect(): void;
}

export class MockServer {
  private readonly pages = new Map<string, PageInfo>();
  private readonly entries = new Map<string, Map<string, Entry>>();
  private readonly clients = new Set<Handler>();

  constructor() {
    this.loadData();
  }

  connect(): MockConnection {
    const handlers = new Set<Handler>();
    const handler: Handler = (message) => {
      for (const listener of handlers) {
        listener(message);
      }
    };

    this.clients.add(handler);

    return {
      send: (message) => {
        this.receive(message, handler);
      },
      subscribe: (listener) => {
        handlers.add(listener);
        return () => {
          handlers.delete(listener);
        };
      },
      disconnect: () => {
        handlers.clear();
        this.clients.delete(handler);
      },
    };
  }

  private loadData(): void {
    for (const page of pages) {
      this.pages.set(page.id, page);
      const pageEntries = new Map<string, Entry>();
      for (const entry of entriesByPage[page.id] ?? []) {
        pageEntries.set(entry.id, structuredClone(entry));
      }
      this.entries.set(page.id, pageEntries);
    }
  }

  private receive(message: ClientMessage, sender: Handler): void {
    switch (message.type) {
      case "connect":
        this.send(sender, { type: "connection_result", result: "connected" });
        this.send(sender, {
          type: "page_tree",
          pages: [...this.pages.values()],
        });
        break;
      case "create_page":
        this.createPage(message.page);
        break;
      case "delete_page":
        this.deletePage(message.page_id);
        break;
      case "edit_page":
        this.editPage(message.page);
        break;
      case "open_page":
        this.openPage(message.page_id, sender);
        break;
      case "close_page":
        break;
      case "create_entry":
        this.createEntry(message.page_id, message.entry_id, message.data);
        break;
      case "delete_entry":
        this.deleteEntry(message.page_id, message.entry_id);
        break;
      case "edit_entry":
        this.editEntry(
          message.page_id,
          message.entry_id,
          message.field,
          message.value,
        );
        break;
      case "publish":
        break;
    }
  }

  private createPage(page: PageInfo): void {
    if (this.pages.has(page.id)) {
      return;
    }
    this.pages.set(page.id, page);
    this.entries.set(page.id, new Map());
    this.broadcast({ type: "page_created", page });
  }

  private deletePage(pageId: string): void {
    if (!this.pages.delete(pageId)) {
      return;
    }
    this.entries.delete(pageId);
    this.broadcast({ type: "page_deleted", page_id: pageId });
  }

  private editPage(page: PageInfo): void {
    if (!this.pages.has(page.id)) {
      return;
    }
    this.pages.set(page.id, page);
    this.broadcast({ type: "page_edited", page });
  }

  private openPage(pageId: string, sender: Handler): void {
    const page = this.pages.get(pageId);
    const entries = this.entries.get(pageId);
    if (!page || !entries) {
      return;
    }
    this.send(sender, {
      type: "page_content",
      page_id: pageId,
      content: JSON.stringify({ page, entries: [...entries.values()] }),
    });
  }

  private createEntry(pageId: string, entryId: string, data: EntryData): void {
    const entries = this.entries.get(pageId);
    if (!entries || entries.has(entryId)) {
      return;
    }
    entries.set(entryId, { id: entryId, ...data });
    this.broadcast({
      type: "entry_created",
      page_id: pageId,
      entry_id: entryId,
      data,
    });
  }

  private deleteEntry(pageId: string, entryId: string): void {
    const entries = this.entries.get(pageId);
    if (!entries?.delete(entryId)) {
      return;
    }
    this.broadcast({
      type: "entry_deleted",
      page_id: pageId,
      entry_id: entryId,
    });
  }

  private editEntry(
    pageId: string,
    entryId: string,
    field: string,
    value: Value,
  ): void {
    const entry = this.entries.get(pageId)?.get(entryId);
    if (!entry) {
      return;
    }
    entry.fields[field] = value;
    this.broadcast({
      type: "entry_edited",
      page_id: pageId,
      entry_id: entryId,
      field,
      value,
    });
  }

  private broadcast(message: ServerMessage): void {
    for (const client of this.clients) {
      this.send(client, message);
    }
  }

  private send(client: Handler, message: ServerMessage): void {
    client(message);
  }
}
