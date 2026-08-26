import { mockEntriesByPageId } from "$lib/mocks/page-content";
import { mockPages } from "$lib/mocks/pages";
import type { Entry, PageInfo } from "$lib/types/model";

class AppStore {
  pages = $state<PageInfo[]>(mockPages);
  selectedPageId = $state<string | null>(mockPages[0]?.id ?? null);
  selectedEntryId = $state<string | null>(null);

  selectedPage = $derived(
    this.pages.find((page) => page.id === this.selectedPageId) ?? null,
  );

  entries = $derived(mockEntriesByPageId[this.selectedPageId ?? ""] ?? []);

  selectedEntry = $derived(
    this.entries.find((entry: Entry) => entry.id === this.selectedEntryId) ??
      null,
  );

  selectPage(id: string) {
    this.selectedPageId = id;
    this.selectedEntryId = null;
  }

  selectEntry(id: string) {
    this.selectedEntryId = id;
  }

  clearEntrySelection() {
    this.selectedEntryId = null;
  }
}

export const appStore = new AppStore();
