import type { Entry, EntryDefinition, PageInfo } from "$lib/types/model";

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
  {
    id: "quest-sequence",
    name: "Quest",
    page_type: "sequence",
    priority: 1,
  },
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
        triggers: {
          list: [{ reference: "01029942073P2:1000002" }],
        },
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
          list: [
            { reference: "01029942073P2:1000003" },
            { reference: "01029942073P2:1000004" },
          ],
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
        triggers: {
          list: [{ reference: "01029942073P2:1000007" }],
        },
      },
    },
    {
      id: "1000004",
      entry_type: "dialogue",
      fields: {
        name: { text: "Accept Quest" },
        text: { text: "Excellent. J'ai justement besoin de ton aide." },
        speaker: { text: "Village Elder" },
        triggers: {
          list: [{ reference: "01029942073P2:1000005" }],
        },
      },
    },
    {
      id: "1000005",
      entry_type: "action",
      fields: {
        name: { text: "Give Quest" },
        action: { text: "first_steps" },
        triggers: {
          list: [{ reference: "01029942073P2:1000006" }],
        },
      },
    },
    {
      id: "1000006",
      entry_type: "dialogue",
      fields: {
        name: { text: "Quest Started" },
        text: { text: "Ton aventure commence maintenant." },
        speaker: { text: "Village Elder" },
        triggers: {
          list: [{ reference: "01029942073P2:1000007" }],
        },
      },
    },
    {
      id: "1000007",
      entry_type: "action",
      fields: {
        name: { text: "Finish Introduction" },
        action: { text: "complete_intro" },
        triggers: {
          list: [{ reference: "01029942073P2:1000008" }],
        },
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
      entry_type: "setting_integer",
      fields: {
        name: { text: "Max Players" },
        value: { integer: 20 },
      },
    },
    {
      id: "2000002",
      entry_type: "setting_text",
      fields: {
        name: { text: "MOTD" },
        value: { text: "Welcome to the server!" },
      },
    },
    {
      id: "2000003",
      entry_type: "setting_boolean",
      fields: {
        name: { text: "PVP" },
        value: { boolean: true },
      },
    },
    {
      id: "2000004",
      entry_type: "setting_enum",
      fields: {
        name: { text: "Difficulty" },
        value: { enum: "normal" },
      },
    },
  ],

  "quest-sequence": [
    {
      id: "4000001",
      entry_type: "quest",
      fields: {
        name: { text: "First Steps" },
        description: {
          text: "La première quête du joueur.",
        },
        introduction: {
          reference: "01029942073P2:1000004",
        },
        previous: { reference: "" },
      },
    },
    {
      id: "4000002",
      entry_type: "quest",
      fields: {
        name: { text: "Village Hero" },
        description: {
          text: "Terminer l'introduction du village.",
        },
        introduction: { reference: "" },
        previous: {
          reference: "quest-sequence:4000001",
        },
      },
    },
  ],

  "game-settings": [
    {
      id: "5000001",
      entry_type: "setting_reference",
      fields: {
        name: { text: "Default Quest" },
        value: {
          reference: "quest-sequence:4000001",
        },
      },
    },
    {
      id: "5000002",
      entry_type: "setting_reference",
      fields: {
        name: { text: "Welcome Dialogue" },
        value: {
          reference: "01029942073P2:1000001",
        },
      },
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

        reference_value: {
          reference: "01029942073P2:1000001",
        },

        struct_value: {
          struct: {
            name: { text: "Test Structure" },
            enabled: { boolean: true },
            priority: { integer: 100 },
            multiplier: { float: 1.5 },
            reference: {
              reference: "server-configuration:2000001",
            },
          },
        },

        list_value: {
          list: [
            { text: "First item" },
            { integer: 42 },
            { boolean: false },
            { float: 12.5 },
            { enum: "example" },
            {
              reference: "01029942073P2:1000002",
            },
            {
              reference: "server-configuration:2000002",
            },
            "null",
            {
              struct: {
                name: { text: "Nested structure" },
                value: { integer: 123 },
                reference: {
                  reference: "quest-sequence:4000001",
                },
              },
            },
            {
              list: [
                { text: "Nested list item" },
                { integer: 999 },
                {
                  reference: "game-settings:5000002",
                },
              ],
            },
          ],
        },
      },
    },
  ],
};

export const entryDefinitions: Record<string, EntryDefinition> = {
  dialogue: {
    entry_type: "dialogue",
    tags: ["sequence"],
    fields: [
      { name: "name", schema: "text" },
      { name: "text", schema: "text" },
      { name: "speaker", schema: "text" },
      {
        name: "triggers",
        schema: { list: { reference: { tags: ["sequence"] } } },
      },
    ],
  },

  action: {
    entry_type: "action",
    tags: ["sequence"],
    fields: [
      { name: "name", schema: "text" },
      { name: "action", schema: "text" },
      {
        name: "triggers",
        schema: { list: { reference: { tags: ["sequence"] } } },
      },
    ],
  },

  quest: {
    entry_type: "quest",
    tags: ["manifest"],
    fields: [
      { name: "name", schema: "text" },
      { name: "description", schema: "text" },
      { name: "introduction", schema: { reference: { tags: ["sequence"] } } },
      {
        name: "previous",
        schema: { reference: { entry_type: "quest", tags: [] } },
      },
    ],
  },

  setting_integer: {
    entry_type: "setting_integer",
    tags: [],
    fields: [
      { name: "name", schema: "text" },
      { name: "value", schema: "integer" },
    ],
  },

  setting_text: {
    entry_type: "setting_text",
    tags: [],
    fields: [
      { name: "name", schema: "text" },
      { name: "value", schema: "text" },
    ],
  },

  setting_boolean: {
    entry_type: "setting_boolean",
    tags: [],
    fields: [
      { name: "name", schema: "text" },
      { name: "value", schema: "boolean" },
    ],
  },

  setting_enum: {
    entry_type: "setting_enum",
    tags: [],
    fields: [
      { name: "name", schema: "text" },
      {
        name: "value",
        schema: { enumeration: ["peaceful", "easy", "normal", "hard"] },
      },
    ],
  },

  setting_reference: {
    entry_type: "setting_reference",
    tags: [],
    fields: [
      { name: "name", schema: "text" },
      { name: "value", schema: { reference: { tags: [] } } },
    ],
  },
};
