import type {
  ConnectionState,
  Entry,
  Page,
  PageConnection,
  PageType,
  Selection,
  Value,
} from "../types";
import { mockPages } from "../mock/pages";
class WorkspaceState {
  pages = $state<Page[]>(structuredClone(mockPages));
  selection = $state<Selection>({ type: "page", pageId: this.pages[0].id });
  connection = $state<ConnectionState>("connecting");
  constructor() {
    setTimeout(() => (this.connection = "connected"), 900);
  }
  get selectedPage() {
    return this.selection
      ? (this.pages.find((p) => p.id === this.selection!.pageId) ?? null)
      : null;
  }
  get selectedEntry() {
    return this.selection?.type === "entry"
      ? (this.selectedPage?.entries.find(
          (e) => e.id === this.selection!.entryId,
        ) ?? null)
      : null;
  }
  selectPage(pageId: string) {
    this.selection = { type: "page", pageId };
  }
  selectEntry(pageId: string, entryId: string) {
    this.selection = { type: "entry", pageId, entryId };
  }
  createPage(name: string, pageType: PageType = "sequence") {
    const id = crypto.randomUUID();
    this.pages.push({
      id,
      name,
      pageType,
      priority: this.pages.length,
      entries: [],
      connections: [],
    });
    this.selectPage(id);
  }
  deleteSelected() {
    if (this.selection?.type === "entry") {
      const p = this.selectedPage;
      if (!p) return;
      const id = this.selection.entryId;
      p.entries = p.entries.filter((e) => e.id !== id);
      p.connections = p.connections.filter(
        (c) => c.source !== id && c.target !== id,
      );
      this.selectPage(p.id);
      return;
    }
    if (this.selection?.type === "page") {
      const id = this.selection.pageId;
      const i = this.pages.findIndex((p) => p.id === id);
      this.pages = this.pages.filter((p) => p.id !== id);
      const next = this.pages[i] ?? this.pages[i - 1] ?? null;
      this.selection = next ? { type: "page", pageId: next.id } : null;
    }
  }
  createEntry() {
    const p = this.selectedPage;
    if (!p) return;
    const id = crypto.randomUUID();
    p.entries.push({
      id,
      data: { entryType: "entry", fields: { name: "New entry" } },
      position: {
        x: 120 + p.entries.length * 35,
        y: 120 + p.entries.length * 35,
      },
    });
    this.selectEntry(p.id, id);
  }
  updatePage(id: string, v: Partial<Pick<Page, "name" | "priority">>) {
    const p = this.pages.find((p) => p.id === id);
    if (p) Object.assign(p, v);
  }
  updateEntryField(
    pageId: string,
    entryId: string,
    field: string,
    value: Value,
  ) {
    const e = this.pages
      .find((p) => p.id === pageId)
      ?.entries.find((e) => e.id === entryId);
    if (e) e.data.fields[field] = value;
  }
  addEntryField(pageId: string, entryId: string) {
    const e = this.pages
      .find((p) => p.id === pageId)
      ?.entries.find((e) => e.id === entryId);
    if (!e) return;
    let n = 1,
      k = "field_1";
    while (k in e.data.fields) k = `field_${++n}`;
    e.data.fields[k] = "";
  }
  updateEntryType(pageId: string, entryId: string, entryType: string) {
    const e = this.pages
      .find((p) => p.id === pageId)
      ?.entries.find((e) => e.id === entryId);
    if (e) e.data.entryType = entryType;
  }
  setConnections(pageId: string, connections: PageConnection[]) {
    const p = this.pages.find((p) => p.id === pageId);
    if (p && p.pageType === "sequence") p.connections = connections;
  }
}
export const workspace = new WorkspaceState();
