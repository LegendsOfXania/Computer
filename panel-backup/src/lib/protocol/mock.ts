import type { Entry, EntryData, PageInfo, Value } from "$lib/types/model";
import { defaultSchema, schemaEqual } from "$lib/types/model";
import type { ClientMessage, ServerMessage } from "./messages";
import { entriesByPage, entryDefinitions, pages } from "./data";
import { mapRefs, removeRefs } from "./refs";

type Handler = (message: ServerMessage) => void;

export interface MockConnection {
  send(message: ClientMessage): void;
  subscribe(handler: Handler): () => void;
  disconnect(): void;
}

function toKey(pageId: string, entryId: string): string {
  return `${pageId}:${entryId}`;
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
        this.send(sender, {
          type: "connection_result",
          result: "connected",
        });

        this.send(sender, {
          type: "library",
          pages: [...this.pages.values()],
          entry_definitions: Object.values(entryDefinitions),
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

      case "get_entry_data":
        this.getEntryData(message.entry_key, sender);
        break;

      case "create_entry":
        this.createEntry(message.entry_key, message.data);
        break;

      case "delete_entry":
        this.deleteEntry(message.entry_key);
        break;

      case "edit_entry":
        this.editEntry(message.entry_key, message.field, message.value);
        break;

      case "move_entry":
        this.moveEntry(message.entry_key, message.target_page_id);
        break;

      case "replace_entry":
        this.replaceEntry(message.entry_key, message.entry_type);
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

    this.broadcast({
      type: "page_created",
      page,
    });
  }

  private deletePage(pageId: string): void {
    if (!this.pages.delete(pageId)) {
      return;
    }

    this.entries.delete(pageId);

    this.broadcast({
      type: "page_deleted",
      page_id: pageId,
    });
  }

  private editPage(page: PageInfo): void {
    if (!this.pages.has(page.id)) {
      return;
    }

    this.pages.set(page.id, page);

    this.broadcast({
      type: "page_edited",
      page,
    });
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
      content: JSON.stringify({
        page,
        entries: [...entries.values()],
      }),
    });
  }

  private getEntryData(entryKey: string, sender: Handler): void {
    const { pageId, entryId } = this.parseEntryKey(entryKey);

    if (!pageId || !entryId) {
      return;
    }

    const entry = this.entries.get(pageId)?.get(entryId);

    if (!entry) {
      return;
    }

    this.send(sender, {
      type: "entry_data",
      entry_key: entryKey,
      data: {
        entry_type: entry.entry_type,
        fields: structuredClone(entry.fields),
      },
    });
  }

  private createEntry(entryKey: string, data: EntryData): void {
    const { pageId, entryId } = this.parseEntryKey(entryKey);

    if (!pageId || !entryId) {
      return;
    }

    const entries = this.entries.get(pageId);

    if (!entries || entries.has(entryId)) {
      return;
    }

    entries.set(entryId, {
      id: entryId,
      ...structuredClone(data),
    });

    this.broadcast({
      type: "entry_created",
      entry_key: entryKey,
      data,
    });
  }

  private deleteEntry(entryKey: string): void {
    const { pageId, entryId } = this.parseEntryKey(entryKey);

    if (!pageId || !entryId) {
      return;
    }

    const entries = this.entries.get(pageId);

    if (!entries?.delete(entryId)) {
      return;
    }

    this.clearRefs(pageId, entryId);

    this.broadcast({
      type: "entry_deleted",
      entry_key: entryKey,
    });
  }

  private clearRefs(pageId: string, entryId: string): void {
    const target = toKey(pageId, entryId);

    for (const [ownerPageId, pageEntries] of this.entries) {
      for (const other of pageEntries.values()) {
        for (const [field, value] of Object.entries(other.fields)) {
          const result = removeRefs(value, (ref) => ref === target);

          if (!result.changed) {
            continue;
          }

          other.fields[field] = result.value;

          this.broadcast({
            type: "entry_edited",
            entry_key: toKey(ownerPageId, other.id),
            field,
            value: result.value,
          });
        }
      }
    }
  }

  private editEntry(entryKey: string, field: string, value: Value): void {
    const { pageId, entryId } = this.parseEntryKey(entryKey);

    if (!pageId || !entryId) {
      return;
    }

    const entry = this.entries.get(pageId)?.get(entryId);

    if (!entry) {
      return;
    }

    entry.fields[field] = value;

    this.broadcast({
      type: "entry_edited",
      entry_key: entryKey,
      field,
      value,
    });
  }

  private moveEntry(entryKey: string, targetPageId: string): void {
    const { pageId, entryId } = this.parseEntryKey(entryKey);

    if (!pageId || !entryId || pageId === targetPageId) {
      return;
    }

    const source = this.entries.get(pageId);
    const target = this.entries.get(targetPageId);
    const entry = source?.get(entryId);

    if (!source || !target || !entry || target.has(entryId)) {
      return;
    }

    source.delete(entryId);
    target.set(entryId, entry);
    this.rewriteRefs(pageId, entryId, targetPageId);

    this.broadcast({
      type: "entry_deleted",
      entry_key: entryKey,
    });

    this.broadcast({
      type: "entry_created",
      entry_key: toKey(targetPageId, entryId),
      data: { entry_type: entry.entry_type, fields: entry.fields },
    });
  }

  private rewriteRefs(
    fromPageId: string,
    entryId: string,
    toPageId: string,
  ): void {
    const source = toKey(fromPageId, entryId);
    const target = toKey(toPageId, entryId);

    for (const [ownerPageId, pageEntries] of this.entries) {
      for (const other of pageEntries.values()) {
        for (const [field, value] of Object.entries(other.fields)) {
          let changed = false;

          const updated = mapRefs(value, (ref) => {
            if (ref !== source) return ref;
            changed = true;
            return target;
          });

          if (!changed) {
            continue;
          }

          other.fields[field] = updated;

          this.broadcast({
            type: "entry_edited",
            entry_key: toKey(ownerPageId, other.id),
            field,
            value: updated,
          });
        }
      }
    }
  }

  private replaceEntry(entryKey: string, entryType: string): void {
    const { pageId, entryId } = this.parseEntryKey(entryKey);

    if (!pageId || !entryId) {
      return;
    }

    const entry = this.entries.get(pageId)?.get(entryId);
    const definition = entryDefinitions[entryType];

    if (!entry || !definition) {
      return;
    }

    const previousDefinition = entryDefinitions[entry.entry_type];
    const fields: Record<string, Value> = {};

    for (const field of definition.fields) {
      const previousField = previousDefinition?.fields.find(
        (f) => f.name === field.name,
      );

      fields[field.name] =
        previousField &&
        schemaEqual(previousField.schema, field.schema) &&
        field.name in entry.fields
          ? entry.fields[field.name]
          : defaultSchema(field.schema);
    }

    entry.entry_type = entryType;
    entry.fields = fields;

    this.broadcast({
      type: "entry_replaced",
      entry_key: entryKey,
      data: { entry_type: entry.entry_type, fields: entry.fields },
    });
  }

  private parseEntryKey(entryKey: string): {
    pageId: string;
    entryId: string;
  } {
    const separator = entryKey.indexOf(":");

    if (separator === -1) {
      return {
        pageId: "",
        entryId: "",
      };
    }

    return {
      pageId: entryKey.slice(0, separator),
      entryId: entryKey.slice(separator + 1),
    };
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
