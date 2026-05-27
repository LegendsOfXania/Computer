import {
  createContext,
  useContext,
  useState,
  useCallback,
  useMemo,
  type ReactNode,
} from 'react'
import type { Tab, Folder, FolderChild, ComputerFile, ConnectionStatus } from '../types'
import { useWebSocket } from './useWebsocket'

interface AppState {
  tabs: Tab[]
  activeTabId: string | null
  openTab: (tab: Omit<Tab, 'dirty'>) => void
  closeTab: (tabId: string) => void
  setActiveTab: (tabId: string) => void
  markDirty: (tabId: string, dirty: boolean) => void

  rootFolders: Folder[]
  allFiles: ComputerFile[]

  connectionStatus: ConnectionStatus

  publishFile: (file: ComputerFile) => void
}

const AppContext = createContext<AppState | null>(null)

function collectFiles(children: FolderChild[]): ComputerFile[] {
  const files: ComputerFile[] = []
  for (const child of children) {
    if (child.kind === 'file') files.push(child.file)
    else files.push(...collectFiles(child.folder.children))
  }
  return files
}

const HOME_TAB: Tab = { id: 'home', kind: 'home', label: 'Home', dirty: false }

export function AppProvider({ children }: { children: ReactNode }) {
  const [tabs, setTabs] = useState<Tab[]>([HOME_TAB])
  const [activeTabId, setActiveTabId] = useState<string | null>('home')
  const [rootFolders, setRootFolders] = useState<Folder[]>([])
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('disconnected')

  const allFiles = useMemo(
    () => rootFolders.flatMap((f) => collectFiles(f.children)),
    [rootFolders],
  )

  const openTab = useCallback((tab: Omit<Tab, 'dirty'>) => {
    setTabs((prev) => {
      if (prev.find((t) => t.id === tab.id)) {
        setActiveTabId(tab.id)
        return prev
      }
      return [...prev, { ...tab, dirty: false }]
    })
    setActiveTabId(tab.id)
  }, [])

  const closeTab = useCallback((tabId: string) => {
    if (tabId === 'home') return
    setTabs((prev) => {
      const idx = prev.findIndex((t) => t.id === tabId)
      const next = prev.filter((t) => t.id !== tabId)
      setActiveTabId((id) => {
        if (id !== tabId) return id
        return next.length ? next[Math.min(idx, next.length - 1)].id : null
      })
      return next
    })
  }, [])

  const markDirty = useCallback((tabId: string, dirty: boolean) => {
    setTabs((prev) => prev.map((t) => (t.id === tabId ? { ...t, dirty } : t)))
  }, [])

  const onSync = useCallback((payload: unknown) => {
    const data = payload as { folders: Folder[] }
    if (Array.isArray(data?.folders)) {
      setRootFolders(data.folders)
    }
  }, [])

  const handlers = useMemo(() => ({ onSync }), [onSync])

  const { send } = useWebSocket(setConnectionStatus, handlers)

  const publishFile = useCallback(
    (file: ComputerFile) => {
      setConnectionStatus('stagging')
      send({ type: 'publish', file })
    },
    [send],
  )

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
        allFiles,
        connectionStatus,
        publishFile,
      }}
    >
      {children}
    </AppContext.Provider>
  )
}

export function useApp() {
  const ctx = useContext(AppContext)
  if (!ctx) throw new Error('useApp must be used within AppProvider')
  return ctx
}
