import { GitBranch, Layers, Film, Database } from "lucide-react";
import type { FileType } from ".";

export const FILE_TYPE_META: Record<
  FileType,
  {
    label: string;
    icon: React.FC<any>;
    color: string;
    desc?: string;
    editorLabel?: string;
  }
> = {
  sequence: {
    label: "Sequence",
    icon: GitBranch,
    color: "var(--type-event)",
    desc: "Triggered chains of actions",
    editorLabel: "Node Editor",
  },
  manifest: {
    label: "Manifest",
    icon: Layers,
    color: "var(--type-fact)",
    desc: "Declarative world state",
    editorLabel: "Graph Editor",
  },
  scene: {
    label: "Scene",
    icon: Film,
    color: "var(--type-dialogue)",
    desc: "Timeline-based cinematics",
    editorLabel: "Timeline Editor",
  },
  static: {
    label: "Static",
    icon: Database,
    color: "var(--type-npc)",
    desc: "Persistent data & facts",
    editorLabel: "List Editor",
  },
};
