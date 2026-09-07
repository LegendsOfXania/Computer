export type SearchQuery = {
  flags: Set<string>;
  values: Map<string, string[]>;
  text: string;
};

const ALIASES: Record<string, string> = {
  e: "entry",
  p: "page",
  ee: "existing",
  n: "new",
  t: "tag",
  ts: "tags",
};

function nameOf(name: string) {
  return ALIASES[name.toLowerCase()] ?? name.toLowerCase();
}

function addValue(values: Map<string, string[]>, name: string, value: string) {
  const list = values.get(name);

  if (list) {
    list.push(value);
  } else {
    values.set(name, [value]);
  }
}

export function valuesOf(query: SearchQuery, name: string) {
  return query.values.get(name) ?? [];
}

export function parseSearchQuery(raw: string): SearchQuery {
  const flags = new Set<string>();
  const values = new Map<string, string[]>();
  const text: string[] = [];

  for (const token of raw.trim().split(/\s+/)) {
    if (!token) continue;

    if (!token.startsWith("!")) {
      text.push(token);
      continue;
    }

    const directive = token.slice(1);
    const index = directive.indexOf(":");

    if (index === -1) {
      flags.add(nameOf(directive));
      continue;
    }

    const name = nameOf(directive.slice(0, index));
    const value = directive.slice(index + 1);

    if (!value) {
      text.push(token);
      continue;
    }

    if (name === "tags") {
      for (const tag of value.split(",")) {
        const trimmed = tag.trim();

        if (trimmed) {
          addValue(values, name, trimmed.toLowerCase());
        }
      }
    } else {
      addValue(values, name, value.toLowerCase());
    }
  }

  return {
    flags,
    values,
    text: text.join(" ").toLowerCase(),
  };
}

export function mergeSearchQueries(...queries: SearchQuery[]): SearchQuery {
  const flags = new Set<string>();
  const values = new Map<string, string[]>();
  const text: string[] = [];

  for (const query of queries) {
    for (const flag of query.flags) {
      flags.add(flag);
    }

    for (const [name, list] of query.values) {
      for (const value of list) {
        addValue(values, name, value);
      }
    }

    if (query.text) {
      text.push(query.text);
    }
  }

  return {
    flags,
    values,
    text: text.join(" "),
  };
}
