import { MockServer } from "$lib/protocol/mock";
import type { ServerMessage } from "$lib/protocol/messages";
import type { Entry, PageInfo, Value } from "$lib/types/model";

const mockServer = new MockServer();

class AppStore {
  private readonly connection = mockServer.connect();

  pages = $state<PageInfo[]>([]);
  selectedPageId = $state<string | null>(null);
  selectedEntryId = $state<string | null>(null);
  entries = $state<Entry[]>([]);

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
    if (!this.selectedPageId) {
      return;
    }

    this.connection.send({
      type: "edit_entry",
      page_id: this.selectedPageId,
      entry_id: entryId,
      field,
      value,
    });
  }

  findEntry(reference: string): Entry | undefined {
    if (!reference) {
      return undefined;
    }

    return this.entries.find((entry) => entry.id === reference);
  }

  private handleMessage(message: ServerMessage) {
    switch (message.type) {
      case "connection_result":
        break;

      case "page_tree":
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

      case "entry_created":
        if (message.page_id === this.selectedPageId) {
          this.entries = [
            ...this.entries,
            {
              id: message.entry_id,
              ...message.data,
            },
          ];
        }
        break;

      case "entry_deleted":
        if (message.page_id === this.selectedPageId) {
          this.entries = this.entries.filter(
            (entry) => entry.id !== message.entry_id,
          );

          if (this.selectedEntryId === message.entry_id) {
            this.selectedEntryId = null;
          }
        }
        break;

      case "entry_edited":
        if (message.page_id === this.selectedPageId) {
          const entry = this.entries.find(
            (entry) => entry.id === message.entry_id,
          );

          if (entry) {
            entry.fields[message.field] = message.value;
          }
        }
        break;
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
    } catch {
      this.entries = [];
    }
  }
}

export const appStore = new AppStore();
