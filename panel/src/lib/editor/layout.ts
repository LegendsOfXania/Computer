import { Position, type Edge, type Node } from "@xyflow/svelte";
import {
  displayName,
  parseEntryReference,
  type Entry,
  type Value,
} from "$lib/types/model";
const X = 260,
  Y = 100,
  COLS = 4;

export function layoutStaticEntries(entries: Entry[]) {
  return {
    nodes: entries.map((e, i) => ({
      id: e.id,
      type: "default",
      position: { x: (i % COLS) * X, y: Math.floor(i / COLS) * Y },
      connectable: false,
      data: { label: displayName(e) },
    })),
    edges: [] as Edge[],
  };
}

export function layoutSequenceEntries(pageId: string, entries: Entry[]) {
  const ids = new Set(entries.map((e) => e.id));
  const outgoing = new Map(
    entries.map((e) => [
      e.id,
      references(pageId, e).filter((id) => ids.has(id)),
    ]),
  );
  const indegree = new Map(entries.map((e) => [e.id, 0]));
  for (const targets of outgoing.values())
    for (const id of targets) indegree.set(id, (indegree.get(id) ?? 0) + 1);

  const rank = new Map<string, number>(),
    queue: string[] = [];
  for (const e of entries)
    if (indegree.get(e.id) === 0) {
      rank.set(e.id, 0);
      queue.push(e.id);
    }
  for (let head = 0; head < queue.length; head++) {
    const id = queue[head],
      current = rank.get(id) ?? 0;
    for (const target of outgoing.get(id) ?? []) {
      rank.set(target, Math.max(rank.get(target) ?? 0, current + 1));
      const left = (indegree.get(target) ?? 0) - 1;
      indegree.set(target, left);
      if (left === 0) queue.push(target);
    }
  }
  for (const e of entries) rank.set(e.id, rank.get(e.id) ?? 0);

  const columns = new Map<number, Entry[]>();
  for (const e of entries) {
    const r = rank.get(e.id)!;
    (columns.get(r) ?? columns.set(r, []).get(r)!).push(e);
  }
  const nodes: Node[] = entries.map((e) => {
    const column = columns.get(rank.get(e.id)!)!,
      index = column.indexOf(e);
    return {
      id: e.id,
      type: "default",
      position: {
        x: (rank.get(e.id) ?? 0) * X,
        y: index * Y - ((column.length - 1) * Y) / 2,
      },
      sourcePosition: Position.Right,
      targetPosition: Position.Left,
      data: { label: displayName(e) },
    };
  });
  const edges: Edge[] = entries.flatMap((e) =>
    (outgoing.get(e.id) ?? []).map((target) => ({
      id: `${e.id}->${target}`,
      source: e.id,
      target,
    })),
  );
  return { nodes, edges };
}
function references(pageId: string, entry: Entry) {
  return Object.values(entry.fields)
    .flatMap((v) => collect(v).map((r) => parseEntryReference(pageId, r)))
    .filter((r) => r.pageId === pageId)
    .map((r) => r.entryId);
}
function collect(value: Value): string[] {
  if (value === "null") return [];
  if ("reference" in value) return value.reference ? [value.reference] : [];
  if ("list" in value) return value.list.flatMap(collect);
  if ("struct" in value) return Object.values(value.struct).flatMap(collect);
  return [];
}
