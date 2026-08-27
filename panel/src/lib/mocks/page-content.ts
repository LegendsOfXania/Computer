import type { Entry } from "$lib/types/model";

export const mockEntriesByPageId: Record<string, Entry[]> = {
  "01029942073P2": [
    {
      id: "1000001",
      entry_type: "dialogue",
      fields: {
        name: { text: "Arrival" },
        text: { text: "Bienvenue dans le village, voyageur." },
        speaker: { text: "Village Elder" },
        triggers: { list: [{ reference: "1000002" }] },
      },
    },
    {
      id: "1000002",
      entry_type: "dialogue",
      fields: {
        name: { text: "Choose Path" },
        text: { text: "Que souhaites-tu faire ?" },
        speaker: { text: "Village Elder" },
        triggers: {
          list: [{ reference: "1000003" }, { reference: "1000004" }],
        },
      },
    },
    {
      id: "1000003",
      entry_type: "dialogue",
      fields: {
        name: { text: "Explore" },
        text: { text: "Très bien. Le village est à toi." },
        speaker: { text: "Village Elder" },
        triggers: { list: [{ reference: "1000007" }] },
      },
    },
    {
      id: "1000004",
      entry_type: "dialogue",
      fields: {
        name: { text: "Accept Quest" },
        text: { text: "Excellent. J'ai justement besoin de ton aide." },
        speaker: { text: "Village Elder" },
        triggers: { list: [{ reference: "1000005" }] },
      },
    },
    {
      id: "1000005",
      entry_type: "action",
      fields: {
        name: { text: "Give Quest" },
        quest: { text: "first_steps" },
        triggers: { list: [{ reference: "1000006" }] },
      },
    },
    {
      id: "1000006",
      entry_type: "dialogue",
      fields: {
        name: { text: "Quest Started" },
        text: { text: "Ton aventure commence maintenant." },
        speaker: { text: "Village Elder" },
        triggers: { list: [{ reference: "1000007" }] },
      },
    },
    {
      id: "1000007",
      entry_type: "action",
      fields: {
        name: { text: "Finish Introduction" },
        action: { text: "complete_intro" },
        triggers: { list: [{ reference: "1000008" }] },
      },
    },
    {
      id: "1000008",
      entry_type: "dialogue",
      fields: {
        name: { text: "Goodbye" },
        text: { text: "Bonne chance, voyageur." },
        speaker: { text: "Village Elder" },
      },
    },
  ],
  "server-configuration": [
    {
      id: "2000001",
      entry_type: "setting",
      fields: {
        name: { text: "Max Players" },
        value: { integer: 20 },
      },
    },
    {
      id: "2000002",
      entry_type: "setting",
      fields: {
        name: { text: "MOTD" },
        value: { text: "Welcome to the server!" },
      },
    },
    {
      id: "2000003",
      entry_type: "setting",
      fields: {
        name: { text: "PVP" },
        value: { boolean: true },
      },
    },
    {
      id: "2000004",
      entry_type: "setting",
      fields: {
        name: { text: "Difficulty" },
        value: { enum: "normal" },
      },
    },
  ],
};
