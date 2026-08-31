import { MockServer } from "$lib/protocol/mock";
import type { ServerMessage } from "$lib/protocol/messages";
import type { Entry, EntryData, PageInfo, Value } from "$lib/types/model";

const mockServer = new MockServer();

class AppStore {
  private readonly connection = mockServer.connect();
  private readonly entryCache = $state(new Map<string, Entry>());

  pages = $state<PageInfo[]>([]);
  entries = $state<Entry[]>([]);

  selectedPageId = $state<string | null>(null);
  selectedEntryId = $state<string | null>(null);

  get selectedPage() {
    return this.pages.find((page) => page.id === this.selectedPageId) ?? null;
  }

  get selectedEntry() {
    return (
      this.entries.find((entry) => entry.id === this.selectedEntryId) ?? null
    );
  }

  constructor() {
    this.connection.subscribe((message) => {
      this.handleMessage(message);
    });

    this.connection.send({
      type: "connect",
      token: "mock-token",
    });
  }

  selectPage(pageId: string) {
    if (pageId === this.selectedPageId) {
      return;
    }

    if (!this.pages.some((page) => page.id === pageId)) {
      return;
    }

    this.selectedPageId = pageId;
    this.selectedEntryId = null;
    this.entries = [];

    this.connection.send({
      type: "open_page",
      page_id: pageId,
    });
  }

  selectEntry(entryId: string) {
    if (this.entries.some((entry) => entry.id === entryId)) {
      this.selectedEntryId = entryId;
    }
  }

  clearEntrySelection() {
    this.selectedEntryId = null;
  }

  updateEntryField(entryId: string, field: string, value: Value) {
    const pageId = this.selectedPageId;

    if (!pageId) {
      return;
    }

    this.connection.send({
      type: "edit_entry",
      entry_key: `${pageId}:${entryId}`,
      field,
      value,
    });
  }

  getEntryData(entryKey: string): Entry | undefined {
    if (!entryKey) {
      return undefined;
    }

    const cached = this.entryCache.get(entryKey);

    if (cached) {
      return cached;
    }

    const [pageId, entryId] = entryKey.split(":");

    if (pageId === this.selectedPageId && entryId) {
      const entry = this.entries.find((entry) => entry.id === entryId);

      if (entry) {
        this.entryCache.set(entryKey, entry);
        return entry;
      }
    }

    this.requestEntry(entryKey);

    return undefined;
  }

  private requestEntry(entryKey: string) {
    if (!entryKey || this.entryCache.has(entryKey)) {
      return;
    }

    this.connection.send({
      type: "get_entry_data",
      entry_key: entryKey,
    });
  }

  private handleMessage(message: ServerMessage) {
    switch (message.type) {
      case "connection_result":
        break;

      case "library":
        this.pages = message.pages;

        if (this.selectedPageId === null && this.pages.length > 0) {
          this.selectPage(this.pages[0].id);
        }
        break;

      case "page_created":
        this.addPage(message.page);
        break;

      case "page_deleted":
        this.removePage(message.page_id);
        break;

      case "page_edited":
        this.updatePage(message.page);
        break;

      case "page_content":
        this.handlePageContent(message.page_id, message.content);
        break;

      case "entry_data":
        this.handleEntryData(message.entry_key, message.data);
        break;

      case "entry_created":
        this.handleEntryCreated(message.entry_key, message.data);
        break;

      case "entry_deleted":
        this.handleEntryDeleted(message.entry_key);
        break;

      case "entry_edited":
        this.handleEntryEdited(message.entry_key, message.field, message.value);
        break;
    }
  }

  private handleEntryData(entryKey: string, data: EntryData) {
    const [, entryId] = entryKey.split(":");

    if (!entryId) {
      return;
    }

    const entry: Entry = {
      id: entryId,
      ...data,
    };

    this.entryCache.set(entryKey, entry);

    const [pageId] = entryKey.split(":");

    if (pageId !== this.selectedPageId) {
      return;
    }

    const index = this.entries.findIndex((item) => item.id === entryId);

    if (index !== -1) {
      this.entries[index] = entry;
    }
  }

  private handleEntryCreated(entryKey: string, data: EntryData) {
    const [pageId, entryId] = entryKey.split(":");

    if (!pageId || !entryId) {
      return;
    }

    const entry: Entry = {
      id: entryId,
      ...data,
    };

    this.entryCache.set(entryKey, entry);

    if (pageId !== this.selectedPageId) {
      return;
    }

    if (this.entries.some((item) => item.id === entryId)) {
      return;
    }

    this.entries = [...this.entries, entry];
  }

  private handleEntryDeleted(entryKey: string) {
    const [pageId, entryId] = entryKey.split(":");

    if (!pageId || !entryId) {
      return;
    }

    this.entryCache.delete(entryKey);

    if (pageId !== this.selectedPageId) {
      return;
    }

    this.entries = this.entries.filter((entry) => entry.id !== entryId);

    if (this.selectedEntryId === entryId) {
      this.selectedEntryId = null;
    }
  }

  private handleEntryEdited(entryKey: string, field: string, value: Value) {
    const [pageId, entryId] = entryKey.split(":");

    if (!pageId || !entryId) {
      return;
    }

    const cached = this.entryCache.get(entryKey);

    if (cached) {
      cached.fields[field] = value;
    }

    if (pageId !== this.selectedPageId) {
      return;
    }

    const entry = this.entries.find((item) => item.id === entryId);

    if (entry) {
      entry.fields[field] = value;
    }
  }

  private addPage(page: PageInfo) {
    if (this.pages.some((existing) => existing.id === page.id)) {
      return;
    }

    this.pages = [...this.pages, page];
  }

  private removePage(pageId: string) {
    this.pages = this.pages.filter((page) => page.id !== pageId);

    for (const key of this.entryCache.keys()) {
      if (key.startsWith(`${pageId}:`)) {
        this.entryCache.delete(key);
      }
    }

    if (this.selectedPageId !== pageId) {
      return;
    }

    this.selectedPageId = null;
    this.selectedEntryId = null;
    this.entries = [];

    const nextPage = this.pages[0];

    if (nextPage) {
      this.selectPage(nextPage.id);
    }
  }

  private updatePage(page: PageInfo) {
    this.pages = this.pages.map((existing) =>
      existing.id === page.id ? page : existing,
    );
  }

  private handlePageContent(pageId: string, content: string) {
    if (pageId !== this.selectedPageId) {
      return;
    }

    try {
      const data = JSON.parse(content) as {
        entries?: Entry[];
      };

      this.entries = data.entries ?? [];

      for (const entry of this.entries) {
        this.entryCache.set(`${pageId}:${entry.id}`, entry);
      }
    } catch {
      this.entries = [];
    }
  }
}

export const appStore = new AppStore();
