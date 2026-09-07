import { createClient } from "$lib/data/client";
import { Repository } from "$lib/data/repository";
import type { ServerMessage } from "$lib/protocol/messages";
import type { Entry, PageInfo, Value } from "$lib/domain/model";
import { LibraryState } from "$lib/state/library.svelte";
import { SelectionState } from "$lib/state/selection.svelte";

class AppStore {
  private readonly client = createClient();
  private readonly repository = new Repository(this.client);
  readonly library = new LibraryState();
  readonly selection = new SelectionState(this.library, this.repository);

  constructor() {
    this.client.subscribe((message) => this.handleMessage(message));
    this.client.send({ type: "connect", token: "mock-token" });
  }

  get pages() { return this.library.pages; }
  get entryDefinitions() { return this.library.entryDefinitions; }
  get selectedPageId() { return this.selection.selectedPageId; }
  set selectedPageId(value: string | null) { this.selection.selectedPageId = value; }
  get selectedPage() { return this.selection.selectedPage; }
  get entries() { return this.selection.entries(); }
  get selectedEntryKey() { return this.selection.selectedKey; }
  get selectedEntryId() { return this.selection.selectedEntryId; }
  get selectedEntry() { return this.selection.selectedEntry; }
  get allEntrySummaries() { return this.library.allSummaries; }

  createPage(name: string, pageType: PageInfo["page_type"], priority: number) {
    this.repository.createPage(name, pageType, priority);
  }

  createEntry(entryType: string, fields: Record<string, Value>): string | null {
    const pageId = this.selectedPageId;
    if (!pageId) return null;
    return this.repository.createEntry(pageId, entryType, fields);
  }

  deleteEntry(entryId: string) {
    const pageId = this.selectedPageId;
    if (pageId) this.repository.deleteEntry(pageId, entryId);
  }

  duplicateEntry(entryId: string) {
    const pageId = this.selectedPageId;
    const entry = pageId ? this.library.getEntry(`${pageId}:${entryId}`) : undefined;
    if (!pageId || !entry) return;
    const key = this.repository.duplicateEntry(pageId, entry);
    this.openReference(key);
  }

  moveEntry(entryId: string, targetPageId: string) {
    if (this.selectedPageId) this.repository.moveEntry(this.selectedPageId, entryId, targetPageId);
  }

  replaceEntry(entryId: string, entryType: string) {
    if (this.selectedPageId) this.repository.replaceEntry(this.selectedPageId, entryId, entryType);
  }

  editPage(pageId: string, name: string, priority: number) {
    const page = this.pages.find((item) => item.id === pageId);
    if (page) this.repository.editPage(page, name, priority);
  }

  removePage(pageId: string) {
    this.repository.deletePage(pageId);
  }

  publish() { this.repository.publish(); }
  selectPage(pageId: string) { this.selection.selectPage(pageId); }
  selectEntry(entryId: string) { this.selection.selectEntry(entryId); }
  openReference(key: string) { this.selection.openReference(key); }
  clearEntrySelection() { this.selection.clearEntrySelection(); }
  requestEntry(key: string) { this.selection.requestEntry(key); }
  getEntryData(key: string): Entry | undefined { return this.library.getEntry(key); }
  fieldSchema(entryType: string, key: string) { return this.library.fieldSchema(entryType, key); }

  updateEntryField(entryId: string, field: string, value: Value) {
    if (this.selectedPageId) this.repository.updateEntryField(this.selectedPageId, entryId, field, value);
  }

  private handleMessage(message: ServerMessage) {
    this.library.handleMessage(message);

    switch (message.type) {
      case "library":
        if (!this.selection.selectedPageId && this.pages.length) this.selection.selectPage(this.pages[0].id);
        break;
      case "page_created":
        this.selection.selectPage(message.page.id);
        break;
      case "page_deleted":
        this.selection.handlePageRemoved(message.page_id);
        break;
      case "page_content":
        this.selection.finishPageLoad(message.page.id);
        break;
      case "entry_deleted":
        if (this.selection.selectedKey === message.entry_key) this.selection.clearEntrySelection();
        break;
    }

  }
}

export const appStore = new AppStore();
