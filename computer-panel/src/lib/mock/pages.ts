import type { Page } from "../types";
export const mockPages: Page[] = [
  {
    id: "introduction",
    name: "Introduction",
    pageType: "sequence",
    priority: 0,
    entries: [
      {
        id: "welcome",
        data: {
          entryType: "dialogue",
          fields: {
            name: "Welcome",
            text: "Welcome to Legends of Xania.",
            speaker: "guide",
          },
        },
        position: { x: 80, y: 180 },
      },
      {
        id: "start",
        data: {
          entryType: "action",
          fields: { name: "Start adventure", action: "begin" },
        },
        position: { x: 430, y: 180 },
      },
      {
        id: "choice",
        data: {
          entryType: "choice",
          fields: { name: "First choice", text: "What will you do?" },
        },
        position: { x: 780, y: 180 },
      },
    ],
    connections: [
      { id: "a", source: "welcome", target: "start" },
      { id: "b", source: "start", target: "choice" },
    ],
  },
  {
    id: "village",
    name: "Village",
    pageType: "sequence",
    priority: 10,
    entries: [
      {
        id: "arrival",
        data: {
          entryType: "dialogue",
          fields: { name: "Arrival", text: "The village is finally in sight." },
        },
        position: { x: 180, y: 180 },
      },
      {
        id: "market",
        data: {
          entryType: "event",
          fields: { name: "Market", event: "open_market" },
        },
        position: { x: 560, y: 180 },
      },
    ],
    connections: [{ id: "c", source: "arrival", target: "market" }],
  },
  {
    id: "facts",
    name: "Player Facts",
    pageType: "static",
    priority: 100,
    entries: [
      {
        id: "reputation",
        data: {
          entryType: "fact",
          fields: { name: "Reputation", type: "integer", default: 0 },
        },
        position: { x: 120, y: 120 },
      },
      {
        id: "completed",
        data: {
          entryType: "fact",
          fields: { name: "Quest completed", type: "boolean", default: false },
        },
        position: { x: 520, y: 320 },
      },
    ],
    connections: [],
  },
];
