import { SvelteMap } from "svelte/reactivity";
import type { ServerMessage } from "$lib/protocol/messages";
import {
  entryKey,
  parseEntryKey,
  summarizeEntry,
  type Entry,
  type EntryDefinition,
  type EntryKey,
  type EntrySummary,
  type Field,
  type PageInfo,
} from "$lib/domain/model";

type RequestState = "idle" | "loading" | "ready" | "error";

export class LibraryState {
  pages = $state<PageInfo[]>([]);
  entryDefinitions = $state<Record<string, EntryDefinition>>({});

  private readonly cache = new SvelteMap<EntryKey, Entry>();

  private readonly order = new SvelteMap<string, string[]>();

  private readonly summaries = new SvelteMap<EntryKey, EntrySummary>();

  private readonly requests = new SvelteMap<EntryKey, RequestState>();

  get allSummaries(): EntrySummary[] {
    return Array.from(this.summaries.values());
  }

  entries(pageId: string | null): Entry[] {
    if (!pageId) return [];

    return (this.order.get(pageId) ?? [])
      .map((id) => this.cache.get(entryKey(pageId, id)))
      .filter((entry): entry is Entry => entry !== undefined);
  }

  getEntry(key: EntryKey): Entry | undefined {
    return this.cache.get(key);
  }

  getEntryRequestState(key: EntryKey): RequestState {
    return this.requests.get(key) ?? "idle";
  }

  fieldSchema(entryType: string, key: string): Field | undefined {
    return this.entryDefinitions[entryType]?.fields.find(
      (field) => field.name === key,
    );
  }

  handleMessage(message: ServerMessage) {
    switch (message.type) {
      case "connection_result":
        break;

      case "library":
        this.pages = message.pages;

        this.entryDefinitions = Object.fromEntries(
          message.entry_definitions.map((definition) => [
            definition.entry_type,
            definition,
          ]),
        );

        this.summaries.clear();

        for (const summary of message.entry_summaries) {
          this.summaries.set(summary.key, summary);
        }

        break;

      case "page_content":
        this.setPageContent(message.page.id, message.entries);
        break;

      case "page_created":
        if (!this.pages.some((page) => page.id === message.page.id)) {
          this.pages = [...this.pages, message.page];
        }
        break;

      case "page_deleted":
        this.removePage(message.page_id);
        break;

      case "page_edited":
        this.pages = this.pages.map((page) =>
          page.id === message.page.id ? message.page : page,
        );
        break;

      case "entry_data":
        this.setEntry(message.entry_key, message.data);
        break;

      case "entry_error":
        this.requests.set(message.entry_key, "error");
        break;

      case "entry_created": {
        const parsed = parseEntryKey(message.entry_key);

        if (!parsed) break;

        this.setEntry(message.entry_key, message.data);

        const ids = this.order.get(parsed.pageId) ?? [];

        if (!ids.includes(parsed.entryId)) {
          this.order.set(parsed.pageId, [...ids, parsed.entryId]);
        }

        this.updateSummary(message.entry_key, {
          id: parsed.entryId,
          ...message.data,
        });

        break;
      }

      case "entry_deleted":
        this.removeEntry(message.entry_key);
        break;

      case "entry_edited": {
        const entry = this.cache.get(message.entry_key);

        if (entry) {
          const updated: Entry = {
            ...entry,
            fields: {
              ...entry.fields,
              [message.field]: message.value,
            },
          };

          this.cache.set(message.entry_key, updated);

          this.updateSummary(message.entry_key, updated);
        }

        break;
      }

      case "entry_replaced": {
        const entry = this.cache.get(message.entry_key);

        if (entry) {
          const updated: Entry = {
            ...entry,
            entry_type: message.data.entry_type,
            fields: message.data.fields,
          };

          this.cache.set(message.entry_key, updated);

          this.updateSummary(message.entry_key, updated);
        }

        break;
      }
    }
  }

  clearPage(pageId: string) {
    this.order.delete(pageId);

    for (const key of this.cache.keys()) {
      const parsed = parseEntryKey(key);

      if (parsed?.pageId !== pageId) continue;

      this.cache.delete(key);
      this.requests.delete(key);
      this.summaries.delete(key);
    }
  }

  removePage(pageId: string) {
    this.pages = this.pages.filter((page) => page.id !== pageId);

    this.clearPage(pageId);
  }

  markLoading(key: EntryKey) {
    this.requests.set(key, "loading");
  }

  private setEntry(key: EntryKey, data: Omit<Entry, "id">) {
    const parsed = parseEntryKey(key);

    if (!parsed) return;

    const entry: Entry = {
      id: parsed.entryId,
      ...data,
    };

    this.cache.set(key, entry);
    this.requests.set(key, "ready");

    this.updateSummary(key, entry);
  }

  private setPageContent(pageId: string, entries: Entry[]) {
    const ids: string[] = [];

    this.clearPage(pageId);

    for (const entry of entries) {
      const key = entryKey(pageId, entry.id);

      this.cache.set(key, entry);
      this.requests.set(key, "ready");

      this.updateSummary(key, entry);
      ids.push(entry.id);
    }

    this.order.set(pageId, ids);
  }

  private updateSummary(key: EntryKey, entry: Entry) {
    const parsed = parseEntryKey(key);

    if (!parsed) return;

    this.summaries.set(
      key,
      summarizeEntry(
        parsed.pageId,
        entry,
        this.entryDefinitions[entry.entry_type],
      ),
    );
  }

  private removeEntry(key: EntryKey) {
    const parsed = parseEntryKey(key);

    if (!parsed) return;

    this.cache.delete(key);
    this.summaries.delete(key);
    this.requests.delete(key);

    const ids = this.order.get(parsed.pageId);

    if (ids) {
      this.order.set(
        parsed.pageId,
        ids.filter((id) => id !== parsed.entryId),
      );
    }
  }
}
