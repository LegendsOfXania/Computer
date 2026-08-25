import { mockPages } from "$lib/mocks/pages";
import type { PageInfo } from "$lib/types/model";

class AppStore {
  pages = $state<PageInfo[]>(mockPages);
  selectedPageId = $state<string | null>(mockPages[0]?.id ?? null);

  selectedPage = $derived(
    this.pages.find((page) => page.id === this.selectedPageId) ?? null,
  );

  selectPage(id: string) {
    this.selectedPageId = id;
  }
}

export const appStore = new AppStore();
