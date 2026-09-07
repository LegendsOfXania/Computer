import { Position, type Edge, type Node } from "@xyflow/svelte";
import {
  displayName,
  parseEntryKey,
  type Entry,
  type Value,
} from "$lib/domain/model";

const X = 260;
const Y = 100;
const COLS = 4;

export type ExternalEntry = {
  key: string;
  label: string;
};

export function layoutStaticEntries(entries: Entry[]) {
  return {
    nodes: entries.map((entry, index) => ({
      id: entry.id,
      type: "default",
      position: { x: (index % COLS) * X, y: Math.floor(index / COLS) * Y },
      connectable: false,
      data: { label: displayName(entry) },
    })),
    edges: [] as Edge[],
    externalKeys: [] as string[],
  };
}

export function layoutSequenceEntries(
  pageId: string,
  entries: Entry[],
  resolveExternal?: (key: string) => string,
) {
  const ids = new Set(entries.map((entry) => entry.id));
  const outgoing = new Map<string, string[]>();
  const external = new Map<string, string[]>();

  for (const entry of entries) {
    const local: string[] = [];
    const ext: string[] = [];
    for (const key of references(entry)) {
      const parsed = parseEntryKey(key);
      if (!parsed) continue;
      if (parsed.pageId === pageId && ids.has(parsed.entryId))
        local.push(parsed.entryId);
      else ext.push(key);
    }
    outgoing.set(entry.id, local);
    external.set(entry.id, ext);
  }

  const indegree = new Map(entries.map((entry) => [entry.id, 0]));
  for (const targets of outgoing.values())
    for (const target of targets)
      indegree.set(target, (indegree.get(target) ?? 0) + 1);

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
      if (left === 0) queue.push(target);
    }
  }
  for (const entry of entries) rank.set(entry.id, rank.get(entry.id) ?? 0);

  const columns = new Map<number, Entry[]>();
  for (const entry of entries) {
    const column = columns.get(rank.get(entry.id)!);
    if (column) column.push(entry);
    else columns.set(rank.get(entry.id)!, [entry]);
  }

  const nodes: Node[] = entries.map((entry) => {
    const column = columns.get(rank.get(entry.id)!)!;
    return {
      id: entry.id,
      type: "default",
      position: {
        x: (rank.get(entry.id) ?? 0) * X,
        y: column.indexOf(entry) * Y - ((column.length - 1) * Y) / 2,
      },
      sourcePosition: Position.Right,
      targetPosition: Position.Left,
      data: { label: displayName(entry) },
    };
  });

  const edges: Edge[] = entries.flatMap((entry) =>
    (outgoing.get(entry.id) ?? []).map((target) => ({
      id: `${entry.id}->${target}`,
      source: entry.id,
      target,
      selectable: false,
    })),
  );

  const maxRank = Math.max(0, ...rank.values());
  const externalX = (maxRank + 1) * X;
  const externalKeys = [...new Set([...external.values()].flat())];
  const externalNodes: Node[] = externalKeys.map((key, index) => ({
    id: key,
    type: "default",
    class: "external",
    position: { x: externalX, y: index * Y },
    targetPosition: Position.Left,
    draggable: false,
    data: { label: resolveExternal?.(key) ?? key },
  }));
  const externalEdges: Edge[] = entries.flatMap((entry) =>
    (external.get(entry.id) ?? []).map((key) => ({
      id: `${entry.id}->${key}`,
      source: entry.id,
      target: key,
      selectable: false,
      style: "stroke-dasharray: 4;",
    })),
  );

  return {
    nodes: [...nodes, ...externalNodes],
    edges: [...edges, ...externalEdges],
    externalKeys,
  };
}

function references(entry: Entry): string[] {
  return Object.values(entry.fields).flatMap(collect);
}

function collect(value: Value): string[] {
  if ("reference" in value) return value.reference ? [value.reference] : [];
  if ("list" in value) return value.list.flatMap(collect);
  if ("struct" in value) return Object.values(value.struct).flatMap(collect);
  return [];
}
