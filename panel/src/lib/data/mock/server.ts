import type { Entry, EntryData, PageInfo, Value } from "$lib/domain/model";
import {
  defaultSchema,
  entryKey,
  parseEntryKey,
  schemaEqual,
  summarizeEntry,
  type EntrySummary,
} from "$lib/domain/model";
import type { ClientMessage, ServerMessage } from "$lib/protocol/messages";
import { entriesByPage, entryDefinitions, pages } from "./data";
import { mapRefs, removeRefs } from "$lib/domain/refs";

type Handler = (message: ServerMessage) => void;

export interface MockConnection {
  send(message: ClientMessage): void;
  subscribe(handler: Handler): () => void;
  disconnect(): void;
}

export interface MockServerOptions {
  latencyMs?: number;
  failureRate?: number;
  timeoutRate?: number;
}

export class MockServer {
  private readonly pages = new Map<string, PageInfo>();
  private readonly entries = new Map<string, Map<string, Entry>>();
  private readonly clients = new Set<Handler>();
  private readonly latencyMs: number;
  private readonly failureRate: number;
  private readonly timeoutRate: number;

  constructor(options: MockServerOptions = {}) {
    this.latencyMs = Math.max(0, options.latencyMs ?? 0);
    this.failureRate = Math.min(1, Math.max(0, options.failureRate ?? 0));
    this.timeoutRate = Math.min(1, Math.max(0, options.timeoutRate ?? 0));
    this.loadData();
  }

  connect(): MockConnection {
    const handlers = new Set<Handler>();
    const handler: Handler = (message) => {
      for (const listener of handlers) listener(message);
    };

    this.clients.add(handler);

    return {
      send: (message) => this.receive(message, handler),
      subscribe: (listener) => {
        handlers.add(listener);
        return () => handlers.delete(listener);
      },
      disconnect: () => {
        handlers.clear();
        this.clients.delete(handler);
      },
    };
  }

  private loadData() {
    for (const page of pages) {
      this.pages.set(page.id, page);

      const pageEntries = new Map<string, Entry>();

      for (const entry of entriesByPage[page.id] ?? []) {
        pageEntries.set(entry.id, structuredClone(entry));
      }

      this.entries.set(page.id, pageEntries);
    }
  }

  private receive(message: ClientMessage, sender: Handler) {
    if (message.type === "connect") {
      this.respond(sender, {
        type: "connection_result",
        result: "connected",
      });

      this.respond(sender, {
        type: "library",
        pages: [...this.pages.values()],
        entry_definitions: Object.values(entryDefinitions),
        entry_summaries: this.buildSummaries(),
      });

      return;
    }

    if (message.type === "get_entry_data" && this.shouldTimeout()) return;

    if (message.type === "get_entry_data" && this.shouldFail()) {
      this.respond(sender, {
        type: "entry_error",
        entry_key: message.entry_key,
        message: "Mock request failed",
      });

      return;
    }

    switch (message.type) {
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

  private respond(sender: Handler, message: ServerMessage) {
    const deliver = () => sender(message);

    if (this.latencyMs > 0) {
      setTimeout(deliver, this.latencyMs);
    } else {
      queueMicrotask(deliver);
    }
  }

  private send(sender: Handler, message: ServerMessage) {
    this.respond(sender, message);
  }

  private broadcast(message: ServerMessage) {
    for (const client of this.clients) {
      this.send(client, message);
    }
  }

  private shouldFail() {
    return Math.random() < this.failureRate;
  }

  private shouldTimeout() {
    return Math.random() < this.timeoutRate;
  }

  private buildSummaries(): EntrySummary[] {
    return [...this.entries].flatMap(([pageId, entries]) =>
      [...entries.values()].map((entry) =>
        summarizeEntry(pageId, entry, entryDefinitions[entry.entry_type]),
      ),
    );
  }

  private createPage(page: PageInfo) {
    if (this.pages.has(page.id)) return;

    this.pages.set(page.id, page);
    this.entries.set(page.id, new Map());

    this.broadcast({
      type: "page_created",
      page,
    });
  }

  private deletePage(pageId: string) {
    if (!this.pages.delete(pageId)) return;

    this.entries.delete(pageId);

    this.broadcast({
      type: "page_deleted",
      page_id: pageId,
    });
  }

  private editPage(page: PageInfo) {
    if (!this.pages.has(page.id)) return;

    this.pages.set(page.id, page);

    this.broadcast({
      type: "page_edited",
      page,
    });
  }

  private openPage(pageId: string, sender: Handler) {
    const page = this.pages.get(pageId);
    const entries = this.entries.get(pageId);

    if (!page || !entries) return;

    this.send(sender, {
      type: "page_content",
      page,
      entries: [...entries.values()].map((entry) => structuredClone(entry)),
    });
  }

  private getEntryData(key: string, sender: Handler) {
    const parsed = parseEntryKey(key);
    if (!parsed) return;

    const entry = this.entries.get(parsed.pageId)?.get(parsed.entryId);

    if (!entry) return;

    this.send(sender, {
      type: "entry_data",
      entry_key: key,
      data: {
        entry_type: entry.entry_type,
        fields: structuredClone(entry.fields),
      },
    });
  }

  private createEntry(key: string, data: EntryData) {
    const parsed = parseEntryKey(key);
    if (!parsed) return;

    const entries = this.entries.get(parsed.pageId);

    if (!entries || entries.has(parsed.entryId)) return;

    entries.set(parsed.entryId, {
      id: parsed.entryId,
      ...structuredClone(data),
    });

    this.broadcast({
      type: "entry_created",
      entry_key: key,
      data: structuredClone(data),
    });
  }

  private deleteEntry(key: string) {
    const parsed = parseEntryKey(key);
    if (!parsed) return;

    const entries = this.entries.get(parsed.pageId);

    if (!entries?.delete(parsed.entryId)) return;

    this.clearRefs(parsed.pageId, parsed.entryId);

    this.broadcast({
      type: "entry_deleted",
      entry_key: key,
    });
  }

  private clearRefs(pageId: string, entryId: string) {
    const target = entryKey(pageId, entryId);

    for (const [ownerPageId, pageEntries] of this.entries) {
      for (const other of pageEntries.values()) {
        for (const [field, value] of Object.entries(other.fields)) {
          const result = removeRefs(value, (ref) => ref === target);

          if (!result.changed) continue;

          other.fields[field] = result.value;

          this.broadcast({
            type: "entry_edited",
            entry_key: entryKey(ownerPageId, other.id),
            field,
            value: result.value,
          });
        }
      }
    }
  }

  private editEntry(key: string, field: string, value: Value) {
    const parsed = parseEntryKey(key);
    if (!parsed) return;

    const entry = this.entries.get(parsed.pageId)?.get(parsed.entryId);

    if (!entry) return;

    entry.fields[field] = structuredClone(value);

    this.broadcast({
      type: "entry_edited",
      entry_key: key,
      field,
      value: structuredClone(value),
    });
  }

  private moveEntry(key: string, targetPageId: string) {
    const parsed = parseEntryKey(key);

    if (!parsed || parsed.pageId === targetPageId) return;

    const source = this.entries.get(parsed.pageId);
    const target = this.entries.get(targetPageId);
    const entry = source?.get(parsed.entryId);

    if (!source || !target || !entry || target.has(parsed.entryId)) {
      return;
    }

    source.delete(parsed.entryId);
    target.set(parsed.entryId, entry);

    this.rewriteRefs(parsed.pageId, parsed.entryId, targetPageId);

    this.broadcast({
      type: "entry_deleted",
      entry_key: key,
    });

    this.broadcast({
      type: "entry_created",
      entry_key: entryKey(targetPageId, parsed.entryId),
      data: {
        entry_type: entry.entry_type,
        fields: structuredClone(entry.fields),
      },
    });
  }

  private rewriteRefs(fromPageId: string, entryId: string, toPageId: string) {
    const source = entryKey(fromPageId, entryId);
    const target = entryKey(toPageId, entryId);

    for (const [ownerPageId, pageEntries] of this.entries) {
      for (const other of pageEntries.values()) {
        for (const [field, value] of Object.entries(other.fields)) {
          let changed = false;

          const updated = mapRefs(value, (ref) => {
            if (ref !== source) return ref;

            changed = true;
            return target;
          });

          if (!changed) continue;

          other.fields[field] = updated;

          this.broadcast({
            type: "entry_edited",
            entry_key: entryKey(ownerPageId, other.id),
            field,
            value: updated,
          });
        }
      }
    }
  }

  private replaceEntry(key: string, entryType: string) {
    const parsed = parseEntryKey(key);
    if (!parsed) return;

    const entry = this.entries.get(parsed.pageId)?.get(parsed.entryId);

    const definition = entryDefinitions[entryType];

    if (!entry || !definition) return;

    const previousDefinition = entryDefinitions[entry.entry_type];

    const fields: Record<string, Value> = {};

    for (const field of definition.fields) {
      const previousField = previousDefinition?.fields.find(
        (item) => item.name === field.name,
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
      entry_key: key,
      data: {
        entry_type: entry.entry_type,
        fields: structuredClone(entry.fields),
      },
    });
  }
}
