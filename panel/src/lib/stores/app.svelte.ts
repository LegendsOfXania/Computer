import { SvelteMap } from "svelte/reactivity";
import { MockServer } from "$lib/protocol/mock";
import type { ServerMessage } from "$lib/protocol/messages";
import type {
  Entry,
  EntryData,
  EntryDefinition,
  Field,
  PageInfo,
  Value,
} from "$lib/types/model";

const mockServer = new MockServer();

function entryKey(pageId: string, entryId: string) {
  return `${pageId}:${entryId}`;
}

class AppStore {
  private readonly connection = mockServer.connect();
  private readonly cache = new SvelteMap<string, Entry>();
  private readonly order = new SvelteMap<string, string[]>();
  private readonly requested = new Set<string>();
  private selectedKey = $state<string | null>(null);
  private pendingSelection: string | null = null;

  pages = $state<PageInfo[]>([]);
  entryDefinitions = $state<Record<string, EntryDefinition>>({});
  selectedPageId = $state<string | null>(null);

  get selectedPage() {
    return this.pages.find((p) => p.id === this.selectedPageId) ?? null;
  }

  get entries(): Entry[] {
    const pageId = this.selectedPageId;
    if (!pageId) return [];

    const ids = this.order.get(pageId) ?? [];
    return ids
      .map((id) => this.cache.get(entryKey(pageId, id)))
      .filter((e): e is Entry => e !== undefined);
  }

  get selectedEntryKey() {
    return this.selectedKey;
  }

  get selectedEntryId() {
    if (!this.selectedKey) return null;
    const [pageId, id] = this.selectedKey.split(":");
    return pageId === this.selectedPageId ? id : null;
  }

  get selectedEntry(): Entry | null {
    return this.selectedKey
      ? (this.getEntryData(this.selectedKey) ?? null)
      : null;
  }

  constructor() {
    this.connection.subscribe((msg) => this.handleMessage(msg));
    this.connection.send({ type: "connect", token: "mock-token" });
  }

  createPage(name: string, pageType: PageInfo["page_type"], priority: number) {
    const pageName = name.trim();
    if (!pageName) return;

    const page: PageInfo = {
      id: crypto.randomUUID(),
      name: pageName,
      page_type: pageType,
      priority,
    };

    this.connection.send({
      type: "create_page",
      page,
    });
  }

  createEntry(entryType: string, fields: Record<string, Value>): string | null {
    const pageId = this.selectedPageId;
    if (!pageId) return null;

    const key = entryKey(pageId, crypto.randomUUID());

    this.connection.send({
      type: "create_entry",
      entry_key: key,
      data: { entry_type: entryType, fields },
    });

    return key;
  }

  editPage(pageId: string, name: string, priority: number) {
    const page = this.pages.find((p) => p.id === pageId);
    if (!page) return;

    this.connection.send({
      type: "edit_page",
      page: { ...page, name, priority },
    });
  }

  selectPage(pageId: string) {
    if (pageId === this.selectedPageId) return;
    if (!this.pages.some((p) => p.id === pageId)) return;

    this.selectedPageId = pageId;
    this.selectedKey = null;
    this.connection.send({ type: "open_page", page_id: pageId });
  }

  selectEntry(entryId: string) {
    if (!this.selectedPageId) return;
    if (!this.entries.some((e) => e.id === entryId)) return;

    this.openReference(entryKey(this.selectedPageId, entryId));
  }

  openReference(key: string) {
    if (!key) return;

    const [pageId] = key.split(":");

    if (pageId !== this.selectedPageId) {
      this.pendingSelection = key;
      this.selectPage(pageId);
      return;
    }

    this.selectedKey = key;
    this.requestEntry(key);
  }

  clearEntrySelection() {
    this.selectedKey = null;
  }

  updateEntryField(entryId: string, field: string, value: Value) {
    const pageId = this.selectedPageId;
    if (!pageId) return;

    this.connection.send({
      type: "edit_entry",
      entry_key: entryKey(pageId, entryId),
      field,
      value,
    });
  }

  getEntryData(key: string): Entry | undefined {
    return key ? this.cache.get(key) : undefined;
  }

  fieldSchema(entryType: string, key: string): Field | undefined {
    return this.entryDefinitions[entryType]?.fields.find(
      (field) => field.name === key,
    );
  }

  requestEntry(key: string) {
    if (!key || this.cache.has(key) || this.requested.has(key)) return;

    this.requested.add(key);
    this.connection.send({ type: "get_entry_data", entry_key: key });
  }

  private handleMessage(message: ServerMessage) {
    switch (message.type) {
      case "library":
        this.pages = message.pages;
        this.entryDefinitions = Object.fromEntries(
          message.entry_definitions.map((d) => [d.entry_type, d]),
        );
        if (!this.selectedPageId && this.pages.length > 0) {
          this.selectPage(this.pages[0].id);
        }
        break;

      case "page_created":
        if (!this.pages.some((p) => p.id === message.page.id)) {
          this.pages = [...this.pages, message.page];
        }

        this.selectPage(message.page.id);
        break;

      case "page_deleted":
        this.removePage(message.page_id);
        break;

      case "page_edited":
        this.pages = this.pages.map((p) =>
          p.id === message.page.id ? message.page : p,
        );
        break;

      case "page_content":
        this.setPageContent(message.page_id, message.content);
        break;

      case "entry_data":
        this.setEntry(message.entry_key, message.data);
        break;

      case "entry_created": {
        const [pageId, entryId] = message.entry_key.split(":");
        this.setEntry(message.entry_key, message.data);
        const ids = this.order.get(pageId);
        if (ids) this.order.set(pageId, [...ids, entryId]);
        break;
      }

      case "entry_deleted":
        this.removeEntry(message.entry_key);
        break;

      case "entry_edited": {
        const entry = this.cache.get(message.entry_key);
        if (entry) entry.fields[message.field] = message.value;
        break;
      }
    }
  }

  private setEntry(key: string, data: EntryData) {
    const [, entryId] = key.split(":");
    if (!entryId) return;

    this.cache.set(key, { id: entryId, ...data });
    this.requested.delete(key);
  }

  private removeEntry(key: string) {
    const [pageId, entryId] = key.split(":");
    this.cache.delete(key);
    this.requested.delete(key);

    const ids = this.order.get(pageId);
    if (ids)
      this.order.set(
        pageId,
        ids.filter((id) => id !== entryId),
      );

    if (this.selectedKey === key) this.selectedKey = null;
  }

  private removePage(pageId: string) {
    this.pages = this.pages.filter((p) => p.id !== pageId);
    this.order.delete(pageId);

    for (const key of this.cache.keys()) {
      if (key.startsWith(`${pageId}:`)) {
        this.cache.delete(key);
        this.requested.delete(key);
      }
    }

    if (this.selectedKey?.startsWith(`${pageId}:`)) {
      this.selectedKey = null;
    }

    if (this.selectedPageId !== pageId) return;

    this.selectedPageId = null;

    const next = this.pages[0];
    if (next) this.selectPage(next.id);
  }

  private setPageContent(pageId: string, content: string) {
    if (pageId !== this.selectedPageId) return;

    try {
      const data = JSON.parse(content) as { entries?: Entry[] };
      const entries = data.entries ?? [];
      const ids: string[] = [];

      for (const entry of entries) {
        this.cache.set(entryKey(pageId, entry.id), entry);
        ids.push(entry.id);
      }

      this.order.set(pageId, ids);
    } catch {
      this.order.set(pageId, []);
    }

    if (this.pendingSelection?.startsWith(`${pageId}:`)) {
      const key = this.pendingSelection;
      this.pendingSelection = null;
      this.selectedKey = key;
      this.requestEntry(key);
    }
  }
}

export const appStore = new AppStore();
