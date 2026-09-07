import type {
  EntryDefinition,
  EntrySummary,
  PageInfo,
} from "$lib/domain/model";
import { formatEntryTypeName, parseEntryKey } from "$lib/domain/model";
import { valuesOf, type SearchQuery } from "./query";

export type SearchEntryResult = EntrySummary;

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
  const normalized = tags.map((tag) => tag.toLowerCase());
  if (
    anyTags.length &&
    !anyTags.some((tag) => normalized.includes(tag.toLowerCase()))
  )
    return false;
  return allTags.every((tag) => normalized.includes(tag.toLowerCase()));
}

function matchesEntryTags(
  query: SearchQuery,
  tags: string[],
  searchEntries: boolean,
  searchPages: boolean,
) {
  const anyTags = [...valuesOf(query, "tag")];
  if (searchEntries && !searchPages) anyTags.push(...valuesOf(query, "type"));
  return matchesTags(tags, anyTags, valuesOf(query, "tags"));
}

export function search(
  query: SearchQuery,
  index: EntrySummary[],
  pages: PageInfo[],
  definitions: Record<string, EntryDefinition>,
): SearchResults {
  const kinds = ["entry", "page"].filter((kind) => query.flags.has(kind));
  const states = ["existing", "new"].filter((state) => query.flags.has(state));
  const searchEntries = kinds.length === 0 || kinds.includes("entry");
  const searchPages = kinds.length === 0 || kinds.includes("page");
  const searchExisting = states.length === 0 || states.includes("existing");
  const searchNew = states.length === 0 || states.includes("new");

  const pageTypes = valuesOf(query, "type");
  const matchingPages =
    searchExisting && searchPages
      ? pages.filter((page) => {
          if (pageTypes.length && !pageTypes.includes(page.page_type))
            return false;
          return matchesText(query.text, page.name, page.id, page.page_type);
        })
      : [];

  const pageNames = new Map(pages.map((page) => [page.id, page.name]));
  const matchingEntries =
    searchExisting && searchEntries
      ? index.filter((summary) => {
          if (
            !matchesEntryTags(query, summary.tags, searchEntries, searchPages)
          )
            return false;
          return matchesText(
            query.text,
            summary.name,
            summary.key,
            summary.entryType,
            pageNames.get(parseEntryKey(summary.key)?.pageId ?? "") ?? "",
          );
        })
      : [];

  const newEntries =
    searchNew && searchEntries
      ? Object.entries(definitions)
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

  return { pages: matchingPages, entries: matchingEntries, newEntries };
}

// entries on the same page should <apparaitre> before ones from an other page
