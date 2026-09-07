import type { EntryKey, PageInfo } from "$lib/domain/model";
import { parseEntryKey } from "$lib/domain/model";
import { LibraryState } from "./library.svelte";
import { Repository } from "$lib/data/repository";

export class SelectionState {
  selectedPageId = $state<string | null>(null);
  selectedKey = $state<EntryKey | null>(null);
  private pendingKey: EntryKey | null = null;

  constructor(
    private readonly library: LibraryState,
    private readonly repository: Repository,
  ) {}

  get selectedPage(): PageInfo | null {
    return (
      this.library.pages.find((page) => page.id === this.selectedPageId) ?? null
    );
  }

  get selectedEntryId(): string | null {
    if (!this.selectedKey) return null;
    const parsed = parseEntryKey(this.selectedKey);
    return parsed?.pageId === this.selectedPageId ? parsed.entryId : null;
  }

  get selectedEntry() {
    return this.selectedKey
      ? (this.library.getEntry(this.selectedKey) ?? null)
      : null;
  }

  entries() {
    return this.library.entries(this.selectedPageId);
  }

  selectPage(pageId: string) {
    if (pageId === this.selectedPageId) return;
    if (!this.library.pages.some((page) => page.id === pageId)) return;

    this.selectedPageId = pageId;
    this.selectedKey = null;
    this.repository.openPage(pageId);
  }

  selectEntry(entryId: string) {
    if (!this.selectedPageId) return;
    if (!this.entries().some((entry) => entry.id === entryId)) return;
    this.openReference(`${this.selectedPageId}:${entryId}`);
  }

  openReference(key: EntryKey) {
    const parsed = parseEntryKey(key);
    if (!parsed) return;

    if (parsed.pageId !== this.selectedPageId) {
      this.pendingKey = key;
      this.selectPage(parsed.pageId);
      return;
    }

    this.selectedKey = key;
    this.requestEntry(key);
  }

  finishPageLoad(pageId: string) {
    if (!this.pendingKey) return;
    const parsed = parseEntryKey(this.pendingKey);
    if (parsed?.pageId !== pageId) return;

    this.selectedKey = this.pendingKey;
    this.pendingKey = null;
    this.requestEntry(this.selectedKey);
  }

  clearEntrySelection() {
    this.selectedKey = null;
  }

  requestEntry(key: EntryKey) {
    if (this.library.getEntry(key)) return;
    if (this.library.getEntryRequestState(key) === "loading") return;
    this.library.markLoading(key);
    this.repository.requestEntry(key);
  }

  handlePageRemoved(pageId: string) {
    if (this.selectedPageId !== pageId) return;
    this.selectedPageId = null;
    this.selectedKey = null;
    const next = this.library.pages[0];
    if (next) this.selectPage(next.id);
  }
}
