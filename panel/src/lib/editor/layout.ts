import { Position, type Edge, type Node } from "@xyflow/svelte";
import { displayName, type Entry, type Value } from "$lib/types/model";

const X = 260;
const Y = 100;
const COLS = 4;

export function layoutStaticEntries(entries: Entry[]) {
  return {
    nodes: entries.map((entry, index) => ({
      id: entry.id,
      type: "default",
      position: {
        x: (index % COLS) * X,
        y: Math.floor(index / COLS) * Y,
      },
      connectable: false,
      data: { label: displayName(entry) },
    })),
    edges: [] as Edge[],
  };
}

export function layoutSequenceEntries(pageId: string, entries: Entry[]) {
  const ids = new Set(entries.map((entry) => entry.id));

  const outgoing = new Map(
    entries.map((entry) => [
      entry.id,
      references(pageId, entry).filter((id) => ids.has(id)),
    ]),
  );

  const indegree = new Map(entries.map((entry) => [entry.id, 0]));

  for (const targets of outgoing.values()) {
    for (const id of targets) {
      indegree.set(id, (indegree.get(id) ?? 0) + 1);
    }
  }

  const rank = new Map<string, number>();
  const queue: string[] = [];

  for (const entry of entries) {
    if (indegree.get(entry.id) === 0) {
      rank.set(entry.id, 0);
      queue.push(entry.id);
    }
  }

  for (let head = 0; head < queue.length; head++) {
    const id = queue[head];
    const current = rank.get(id) ?? 0;

    for (const target of outgoing.get(id) ?? []) {
      rank.set(target, Math.max(rank.get(target) ?? 0, current + 1));

      const left = (indegree.get(target) ?? 0) - 1;
      indegree.set(target, left);

      if (left === 0) {
        queue.push(target);
      }
    }
  }

  for (const entry of entries) {
    rank.set(entry.id, rank.get(entry.id) ?? 0);
  }

  const columns = new Map<number, Entry[]>();

  for (const entry of entries) {
    const rankIndex = rank.get(entry.id)!;

    if (!columns.has(rankIndex)) {
      columns.set(rankIndex, []);
    }

    columns.get(rankIndex)!.push(entry);
  }

  const nodes: Node[] = entries.map((entry) => {
    const column = columns.get(rank.get(entry.id)!)!;
    const index = column.indexOf(entry);

    return {
      id: entry.id,
      type: "default",
      position: {
        x: (rank.get(entry.id) ?? 0) * X,
        y: index * Y - ((column.length - 1) * Y) / 2,
      },
      sourcePosition: Position.Right,
      targetPosition: Position.Left,
      data: {
        label: displayName(entry),
      },
    };
  });

  const edges: Edge[] = entries.flatMap((entry) =>
    (outgoing.get(entry.id) ?? []).map((target) => ({
      id: `${entry.id}->${target}`,
      source: entry.id,
      target,
    })),
  );

  return { nodes, edges };
}

function references(pageId: string, entry: Entry): string[] {
  return Object.values(entry.fields)
    .flatMap((value) => collect(value))
    .filter((entryId) => {
      return entryId.length > 0;
    });
}

function collect(value: Value): string[] {
  if (value === "null") {
    return [];
  }

  if ("reference" in value) {
    return value.reference ? [value.reference] : [];
  }

  if ("list" in value) {
    return value.list.flatMap(collect);
  }

  if ("struct" in value) {
    return Object.values(value.struct).flatMap(collect);
  }

  return [];
}
