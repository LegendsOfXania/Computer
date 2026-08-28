import { entriesByPage, pages } from "$lib/data";
import type { Entry, PageInfo, Value } from "$lib/types/model";

const clone = <T>(value: T): T => structuredClone(value);

class AppStore {
  pages = $state<PageInfo[]>(clone(pages));
  selectedPageId = $state<string | null>(pages[0]?.id ?? null);
  selectedEntryId = $state<string | null>(null);
  entries = $state<Entry[]>(clone(entriesByPage[pages[0]?.id ?? ""] ?? []));

  get selectedPage() {
    return this.pages.find((p) => p.id === this.selectedPageId) ?? null;
  }
  get selectedEntry() {
    return this.entries.find((e) => e.id === this.selectedEntryId) ?? null;
  }

  selectPage(id: string) {
    if (id === this.selectedPageId) return;
    this.selectedPageId = id;
    this.selectedEntryId = null;
    this.entries = clone(entriesByPage[id] ?? []);
  }
  selectEntry(id: string) {
    this.selectedEntryId = id;
  }
  clearEntrySelection() {
    this.selectedEntryId = null;
  }
  updateEntryField(entryId: string, key: string, value: Value) {
    const entry = this.entries.find((e) => e.id === entryId);
    if (entry) entry.fields[key] = value;
  }
  findEntry(reference: string): Entry | undefined {
    if (!reference) return;
    const [pageId, entryId] = reference.includes(":")
      ? reference.split(/:(.*)/s)
      : [this.selectedPageId, reference];
    const source =
      pageId === this.selectedPageId
        ? this.entries
        : entriesByPage[pageId ?? ""];
    return source?.find((e) => e.id === entryId);
  }
}
export const appStore = new AppStore();
