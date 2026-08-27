import { Position, type Edge, type Node } from "@xyflow/svelte";
import {
  displayName,
  parseEntryReference,
  type Entry,
  type Value,
} from "$lib/types/model";

const RANK_GAP_X = 260;
const NODE_GAP_Y = 100;
const GRID_COLUMNS = 4;

export function layoutStaticEntries(entries: Entry[]): {
  nodes: Node[];
  edges: Edge[];
} {
  const nodes: Node[] = entries.map((entry, index) => ({
    id: entry.id,
    type: "default",
    position: {
      x: (index % GRID_COLUMNS) * RANK_GAP_X,
      y: Math.floor(index / GRID_COLUMNS) * NODE_GAP_Y,
    },
    connectable: false,
    data: { label: displayName(entry) },
  }));

  return { nodes, edges: [] };
}

export function layoutSequenceEntries(
  pageId: string,
  entries: Entry[],
): { nodes: Node[]; edges: Edge[] } {
  const byId = new Map(entries.map((entry) => [entry.id, entry]));

  const outgoing = new Map<string, string[]>();
  const indegree = new Map<string, number>();
  for (const entry of entries) {
    indegree.set(entry.id, 0);
  }

  for (const entry of entries) {
    const targets = referencedEntryIds(pageId, entry).filter((id) =>
      byId.has(id),
    );
    outgoing.set(entry.id, targets);
    for (const target of targets) {
      indegree.set(target, (indegree.get(target) ?? 0) + 1);
    }
  }

  const rank = new Map<string, number>();
  const remaining = new Map(indegree);
  const queue: string[] = [];

  for (const entry of entries) {
    if (indegree.get(entry.id) === 0) {
      rank.set(entry.id, 0);
      queue.push(entry.id);
    }
  }

  while (queue.length > 0) {
    const id = queue.shift()!;
    const currentRank = rank.get(id) ?? 0;

    for (const target of outgoing.get(id) ?? []) {
      rank.set(target, Math.max(rank.get(target) ?? 0, currentRank + 1));

      const left = (remaining.get(target) ?? 0) - 1;
      remaining.set(target, left);
      if (left === 0) {
        queue.push(target);
      }
    }
  }

  for (const entry of entries) {
    if (!rank.has(entry.id)) {
      rank.set(entry.id, 0);
    }
  }

  const columns = new Map<number, string[]>();
  for (const entry of entries) {
    const entryRank = rank.get(entry.id) ?? 0;
    const column = columns.get(entryRank) ?? [];
    column.push(entry.id);
    columns.set(entryRank, column);
  }

  const nodes: Node[] = entries.map((entry) => {
    const entryRank = rank.get(entry.id) ?? 0;
    const column = columns.get(entryRank) ?? [];
    const indexInColumn = column.indexOf(entry.id);
    const columnHeight = (column.length - 1) * NODE_GAP_Y;

    return {
      id: entry.id,
      type: "default",
      position: {
        x: entryRank * RANK_GAP_X,
        y: indexInColumn * NODE_GAP_Y - columnHeight / 2,
      },
      sourcePosition: Position.Right,
      targetPosition: Position.Left,
      data: { label: displayName(entry) },
    };
  });

  const edges: Edge[] = entries.flatMap((entry) =>
    (outgoing.get(entry.id) ?? []).map((targetId) => ({
      id: `${entry.id}->${targetId}`,
      source: entry.id,
      target: targetId,
    })),
  );

  return { nodes, edges };
}

function referencedEntryIds(pageId: string, entry: Entry): string[] {
  const targets: string[] = [];

  for (const field of Object.values(entry.fields)) {
    for (const reference of collectReferences(field)) {
      const ref = parseEntryReference(pageId, reference);
      if (ref.pageId === pageId) {
        targets.push(ref.entryId);
      }
    }
  }

  return targets;
}

function collectReferences(value: Value): string[] {
  if (typeof value !== "object" || value === null) {
    return [];
  }
  if ("reference" in value) {
    return [value.reference];
  }
  if ("list" in value) {
    return value.list.flatMap(collectReferences);
  }
  if ("struct" in value) {
    return Object.values(value.struct).flatMap(collectReferences);
  }
  return [];
}
