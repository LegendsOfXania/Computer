import type { Entry, PageInfo } from "$lib/types/model";
export const pages: PageInfo[] = [
  {
    id: "01029942073P2",
    name: "Main Sequence",
    page_type: "sequence",
    priority: 10,
  },
  {
    id: "server-configuration",
    name: "Server Configuration",
    page_type: "static",
    priority: 200,
  },
  { id: "quest-sequence", name: "Quest", page_type: "sequence", priority: 1 },
  {
    id: "game-settings",
    name: "Game Settings",
    page_type: "static",
    priority: 1,
  },
  {
    id: "value-types",
    name: "Value Types",
    page_type: "static",
    priority: 999,
  },
];
export const entriesByPage: Record<string, Entry[]> = {
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
      fields: { name: { text: "Max Players" }, value: { integer: 20 } },
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
      fields: { name: { text: "PVP" }, value: { boolean: true } },
    },
    {
      id: "2000004",
      entry_type: "setting",
      fields: { name: { text: "Difficulty" }, value: { enum: "normal" } },
    },
  ],
  "value-types": [
    {
      id: "3000001",
      entry_type: "value_test",
      fields: {
        null_value: "null",
        float_value: { float: 3.141592 },
        integer_value: { integer: 42 },
        boolean_value: { boolean: true },
        text_value: { text: "Hello, Computer!" },
        enum_value: { enum: "production" },
        reference_value: { reference: "1000001" },
        struct_value: {
          struct: {
            name: { text: "Test Structure" },
            enabled: { boolean: true },
            priority: { integer: 100 },
            multiplier: { float: 1.5 },
          },
        },
        list_value: {
          list: [
            { text: "First item" },
            { integer: 42 },
            { boolean: false },
            { float: 12.5 },
            { enum: "example" },
            { reference: "1000002" },
            "null",
            {
              struct: {
                name: { text: "Nested structure" },
                value: { integer: 123 },
              },
            },
            { list: [{ text: "Nested list item" }, { integer: 999 }] },
          ],
        },
      },
    },
  ],
};
