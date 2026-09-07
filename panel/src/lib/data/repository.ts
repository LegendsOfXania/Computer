import { removeRefs } from "$lib/domain/refs";
import {
  entryKey,
  type Entry,
  type EntryData,
  type EntryKey,
  type PageInfo,
  type Value,
} from "$lib/domain/model";
import type { Client } from "./client";

export class Repository {
  constructor(private readonly client: Client) {}

  createPage(name: string, pageType: PageInfo["page_type"], priority: number) {
    const trimmed = name.trim();
    if (!trimmed) return;

    this.client.send({
      type: "create_page",
      page: {
        id: crypto.randomUUID(),
        name: trimmed,
        page_type: pageType,
        priority,
      },
    });
  }

  createEntry(pageId: string, entryType: string, fields: Record<string, Value>): EntryKey {
    const key = entryKey(pageId, crypto.randomUUID());
    this.client.send({
      type: "create_entry",
      entry_key: key,
      data: { entry_type: entryType, fields },
    });
    return key;
  }

  deleteEntry(pageId: string, entryId: string) {
    this.client.send({ type: "delete_entry", entry_key: entryKey(pageId, entryId) });
  }

  duplicateEntry(pageId: string, entry: Entry): EntryKey {
    const fields = Object.fromEntries(
      Object.entries(structuredClone(entry.fields)).map(([field, value]) => [
        field,
        removeRefs(value, () => true).value,
      ]),
    );
    return this.createEntry(pageId, entry.entry_type, fields);
  }

  moveEntry(pageId: string, entryId: string, targetPageId: string) {
    if (pageId === targetPageId) return;
    this.client.send({
      type: "move_entry",
      entry_key: entryKey(pageId, entryId),
      target_page_id: targetPageId,
    });
  }

  replaceEntry(pageId: string, entryId: string, entryType: string) {
    this.client.send({
      type: "replace_entry",
      entry_key: entryKey(pageId, entryId),
      entry_type: entryType,
    });
  }

  editPage(page: PageInfo, name: string, priority: number) {
    this.client.send({
      type: "edit_page",
      page: { ...page, name, priority },
    });
  }

  deletePage(pageId: string) {
    this.client.send({ type: "delete_page", page_id: pageId });
  }

  openPage(pageId: string) {
    this.client.send({ type: "open_page", page_id: pageId });
  }

  requestEntry(key: EntryKey) {
    this.client.send({ type: "get_entry_data", entry_key: key });
  }

  updateEntryField(pageId: string, entryId: string, field: string, value: Value) {
    this.client.send({
      type: "edit_entry",
      entry_key: entryKey(pageId, entryId),
      field,
      value,
    });
  }

  publish() {
    this.client.send({ type: "publish" });
  }
}
