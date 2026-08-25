import type { Page } from "$lib/types/pages";

export const mockPages: Page[] = [
  {
    id: "01029942073P2",
    name: "Main Sequence",
    page_type: "sequence",
    priority: 10,
  },
  {
    id: "quest-sequence",
    name: "Quest",
    page_type: "sequence",
    priority: 1,
  },
  {
    id: "server-configuration",
    name: "Server Configuration",
    page_type: "static",
    priority: 1,
  },
  {
    id: "game-settings",
    name: "Game Settings",
    page_type: "static",
    priority: 1,
  },
];
