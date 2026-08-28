import { mockEntriesByPageId } from "$lib/mocks/page-content";
import { mockPages } from "$lib/mocks/pages";
import type { Entry, PageInfo, Value } from "$lib/types/model";

function cloneEntries(entries: Entry[]): Entry[] {
  return structuredClone(entries);
}

class AppStore {
  pages = $state<PageInfo[]>(mockPages);
  selectedPageId = $state<string | null>(mockPages[0]?.id ?? null);
  selectedEntryId = $state<string | null>(null);
  entries = $state<Entry[]>(
    cloneEntries(mockEntriesByPageId[mockPages[0]?.id ?? ""] ?? []),
  );

  selectedPage = $derived(
    this.pages.find((page) => page.id === this.selectedPageId) ?? null,
  );

  selectedEntry = $derived(
    this.entries.find((entry: Entry) => entry.id === this.selectedEntryId) ??
      null,
  );

  selectPage(id: string) {
    this.selectedPageId = id;
    this.selectedEntryId = null;
    this.entries = cloneEntries(mockEntriesByPageId[id] ?? []);
  }

  selectEntry(id: string) {
    this.selectedEntryId = id;
  }

  clearEntrySelection() {
    this.selectedEntryId = null;
  }

  updateEntryField(entryId: string, fieldKey: string, value: Value) {
    const entry = this.entries.find((e) => e.id === entryId);
    if (entry !== undefined) {
      entry.fields[fieldKey] = value;
    }
  }

  findEntry(reference: string): Entry | undefined {
    if (!reference) return undefined;

    const separatorIndex = reference.indexOf(":");
    if (separatorIndex !== -1) {
      const pageId = reference.slice(0, separatorIndex);
      const entryId = reference.slice(separatorIndex + 1);

      if (pageId === this.selectedPageId) {
        const local = this.entries.find((e) => e.id === entryId);
        if (local !== undefined) return local;
      }

      return mockEntriesByPageId[pageId]?.find((e) => e.id === entryId);
    }

    const local = this.entries.find((e) => e.id === reference);
    if (local !== undefined) return local;

    for (const pageEntries of Object.values(mockEntriesByPageId)) {
      const found = pageEntries.find((e) => e.id === reference);
      if (found !== undefined) return found;
    }

    return undefined;
  }
}

export const appStore = new AppStore();
