import { appStore } from "$lib/stores/app.svelte";
import {
  formatEntryTypeName,
  type Entry,
  type PageInfo,
} from "$lib/types/model";
import { parseSearchQuery, valuesOf, type SearchQuery } from "./query";

export type SearchEntryResult = {
  key: string;
  pageId: string;
  entry: Entry;
};

export type SearchResults = {
  pages: PageInfo[];
  entries: SearchEntryResult[];
  newEntries: string[];
};

function matchesText(query: string, ...values: string[]) {
  if (!query) return true;

  return values.join(" ").toLowerCase().includes(query);
}

function matchesTags(tags: string[], anyTags: string[], allTags: string[]) {
  const result = tags.map((tag) => tag.toLowerCase());

  if (
    anyTags.length &&
    !anyTags.some((tag) => result.includes(tag.toLowerCase()))
  ) {
    return false;
  }

  return allTags.every((tag) => result.includes(tag.toLowerCase()));
}

function matchesEntryTags(
  query: SearchQuery,
  tags: string[],
  searchEntries: boolean,
  searchPages: boolean,
) {
  const anyTags = valuesOf(query, "tag");

  if (searchEntries && !searchPages) {
    anyTags.push(...valuesOf(query, "type"));
  }

  return matchesTags(tags, anyTags, valuesOf(query, "tags"));
}

export function search(query: SearchQuery): SearchResults {
  const kinds = ["entry", "page"].filter((kind) => query.flags.has(kind));

  const states = ["existing", "new"].filter((state) => query.flags.has(state));

  const searchEntries = kinds.length === 0 || kinds.includes("entry");

  const searchPages = kinds.length === 0 || kinds.includes("page");

  const searchExisting = states.length === 0 || states.includes("existing");

  const searchNew = states.length === 0 || states.includes("new");

  const pages =
    searchExisting && searchPages
      ? appStore.pages.filter((page) => {
          const types = valuesOf(query, "type");

          if (types.length && !types.includes(page.page_type)) {
            return false;
          }

          return matchesText(query.text, page.name, page.id, page.page_type);
        })
      : [];

  const entries =
    searchExisting && searchEntries
      ? appStore.allCachedEntries.filter(({ pageId, entry }) => {
          const definition = appStore.entryDefinitions[entry.entry_type];

          if (
            !matchesEntryTags(
              query,
              definition?.tags ?? [],
              searchEntries,
              searchPages,
            )
          ) {
            return false;
          }

          const page = appStore.pages.find((page) => page.id === pageId);

          const name =
            entry.fields.name && "text" in entry.fields.name
              ? entry.fields.name.text
              : entry.id;

          return matchesText(
            query.text,
            name,
            entry.id,
            entry.entry_type,
            page?.name ?? "",
            pageId,
          );
        })
      : [];

  const newEntries =
    searchNew && searchEntries
      ? Object.entries(appStore.entryDefinitions)
          .filter(
            ([type, definition]) =>
              matchesEntryTags(
                query,
                definition.tags,
                searchEntries,
                searchPages,
              ) && matchesText(query.text, type, formatEntryTypeName(type)),
          )
          .map(([type]) => type)
      : [];

  return {
    pages,
    entries,
    newEntries,
  };
}
