import {
  createContext,
  useContext,
  useState,
  useCallback,
  type ReactNode,
} from "react";
import type {
  Tab,
  Folder,
  FolderChild,
  ComputerFile,
  ConnectionStatus,
  Registry,
} from "../types";

interface AppState {
  tabs: Tab[];
  activeTabId: string | null;
  openTab: (tab: Omit<Tab, "dirty">) => void;
  closeTab: (tabId: string) => void;
  setActiveTab: (tabId: string) => void;
  markDirty: (tabId: string, dirty: boolean) => void;

  rootFolders: Folder[];
  setRootFolders: (folders: Folder[]) => void;
  allFiles: ComputerFile[];

  connectionStatus: ConnectionStatus;
  setConnectionStatus: (s: ConnectionStatus) => void;

  registry: Registry | null;
  setRegistry: (r: Registry) => void;
}

const AppContext = createContext<AppState | null>(null);

function collectFiles(children: FolderChild[]): ComputerFile[] {
  const files: ComputerFile[] = [];
  for (const child of children) {
    if (child.kind === "file") files.push(child.file);
    else files.push(...collectFiles(child.folder.children));
  }
  return files;
}

const HOME_TAB: Tab = { id: "home", kind: "home", label: "Home", dirty: false };

export function AppProvider({ children }: { children: ReactNode }) {
  const [tabs, setTabs] = useState<Tab[]>([HOME_TAB]);
  const [activeTabId, setActiveTabId] = useState<string | null>("home");
  const [rootFolders, setRootFolders] = useState<Folder[]>(MOCK_FOLDERS);
  const [connectionStatus, setConnectionStatus] =
    useState<ConnectionStatus>("disconnected");
  const [registry, setRegistry] = useState<Registry | null>(null);

  const allFiles = rootFolders.flatMap((f) => collectFiles(f.children));

  const openTab = useCallback((tab: Omit<Tab, "dirty">) => {
    setTabs((prev) => {
      if (prev.find((t) => t.id === tab.id)) {
        setActiveTabId(tab.id);
        return prev;
      }
      return [...prev, { ...tab, dirty: false }];
    });
    setActiveTabId(tab.id);
  }, []);

  const closeTab = useCallback((tabId: string) => {
    if (tabId === "home") return;
    setTabs((prev) => {
      const idx = prev.findIndex((t) => t.id === tabId);
      const next = prev.filter((t) => t.id !== tabId);
      setActiveTabId((id) => {
        if (id !== tabId) return id;
        if (next.length === 0) return null;
        return next[Math.min(idx, next.length - 1)].id;
      });
      return next;
    });
  }, []);

  const markDirty = useCallback((tabId: string, dirty: boolean) => {
    setTabs((prev) => prev.map((t) => (t.id === tabId ? { ...t, dirty } : t)));
  }, []);

  return (
    <AppContext.Provider
      value={{
        tabs,
        activeTabId,
        openTab,
        closeTab,
        setActiveTab: setActiveTabId,
        markDirty,
        rootFolders,
        setRootFolders,
        allFiles,
        connectionStatus,
        setConnectionStatus,
        registry,
        setRegistry,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}

const MOCK_FOLDERS: Folder[] = [
  {
    id: "folder_npcs",
    name: "npcs",
    children: [
      {
        kind: "folder",
        folder: {
          id: "folder_merchants",
          name: "merchants",
          children: [
            {
              kind: "file",
              file: {
                id: "file_001",
                name: "boucher_activity",
                type: "sequence",
                priority: 0,
                version: "1.0.0",
                entries: [
                  {
                    id: "e_001",
                    name: "boucher_interact_event",
                    type: "entity_interact_event",
                    fields: {
                      triggers: ["e_002"],
                      criteria: [],
                      modifiers: [],
                    },
                  },
                  {
                    id: "e_002",
                    name: "boucher_dialogue",
                    type: "action_bar_dialogue",
                    fields: {
                      triggers: [],
                      criteria: [],
                      modifiers: [],
                      text: "Des porcs, des boeufs !",
                      duration: 1000,
                    },
                  },
                ],
              },
            },
            {
              kind: "file",
              file: {
                id: "file_002",
                name: "boulanger_activity",
                type: "sequence",
                priority: 0,
                version: "1.0.0",
                entries: [],
              },
            },
          ],
        },
      },
      {
        kind: "file",
        file: {
          id: "file_003",
          name: "world_npcs",
          type: "manifest",
          priority: 0,
          version: "1.0.0",
          entries: [],
        },
      },
    ],
  },
  {
    id: "folder_quests",
    name: "quests",
    children: [
      {
        kind: "file",
        file: {
          id: "file_004",
          name: "main_intro",
          type: "sequence",
          priority: 1,
          version: "1.0.0",
          entries: [],
        },
      },
      {
        kind: "file",
        file: {
          id: "file_005",
          name: "intro_cinematic",
          type: "scene",
          priority: 0,
          version: "1.0.0",
          entries: [],
        },
      },
    ],
  },
  {
    id: "folder_data",
    name: "data",
    children: [
      {
        kind: "file",
        file: {
          id: "file_006",
          name: "server_facts",
          type: "static",
          priority: 0,
          version: "1.0.0",
          entries: [],
        },
      },
    ],
  },
];
