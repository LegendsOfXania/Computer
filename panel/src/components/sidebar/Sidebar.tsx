import { useState } from "react";
import {
  Folder,
  FolderOpen,
  FileText,
  Plus,
  Search,
  ChevronRight,
  Upload,
  Unplug,
  Settings,
  Home,
  Loader,
  CloudUpload,
} from "lucide-react";
import { useApp } from "../../store/AppContext";
import { FILE_TYPE_META } from "../../types/fileTypes";
import type {
  FolderChild,
  FileType,
  Folder as FolderType,
  ConnectionStatus,
} from "../../types";
import "./Sidebar.css";

export function Sidebar() {
  const { openTab, activeTabId, connectionStatus, rootFolders } = useApp();
  const [search, setSearch] = useState("");

  return (
    <aside className="sidebar">
      <div className="sidebar__header">
        <span className="sidebar__title">Computer</span>
      </div>

      <div className="sidebar__search">
        <Search size={12} className="sidebar__search-icon" />
        <div className="sidebar__search-placeholder" />
        <input
          className="sidebar__search-input"
          placeholder="Search files…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      <nav className="sidebar__tree" aria-label="Files">
        {rootFolders.map((folder) => (
          <FolderNode
            key={folder.id}
            folder={folder}
            depth={0}
            search={search}
            activeTabId={activeTabId}
            onOpenFile={(file) =>
              openTab({
                id: file.id,
                kind: "file",
                label: file.name,
                fileId: file.id,
              })
            }
          />
        ))}
      </nav>

      <div className="sidebar__footer">
        <button
          className="sidebar__footer-btn"
          onClick={() => openTab({ id: "home", kind: "home", label: "Home" })}
        >
          <Home size={13} />
          <span className="sidebar__label">Home</span>
        </button>
        <button className="sidebar__footer-btn">
          <Plus size={13} />
          <span className="sidebar__label">New file</span>
        </button>
        <button className="sidebar__footer-btn">
          <Settings size={13} />
          <span className="sidebar__label">Settings</span>
        </button>
        <PublishButton status={connectionStatus} />
      </div>
    </aside>
  );
}

function FolderNode({
  folder,
  depth,
  search,
  activeTabId,
  onOpenFile,
}: {
  folder: FolderType;
  depth: number;
  search: string;
  activeTabId: string | null;
  onOpenFile: (file: any) => void;
}) {
  const [open, setOpen] = useState(depth === 0);

  const matchesSearch = (child: FolderChild): boolean => {
    if (!search) return true;
    if (child.kind === "file")
      return child.file.name.toLowerCase().includes(search.toLowerCase());
    return child.folder.children.some(matchesSearch);
  };

  const visible = folder.children.filter(matchesSearch);
  if (search && visible.length === 0) return null;

  return (
    <div
      className="tree-folder"
      style={{ ["--depth" as any]: depth } as React.CSSProperties}
    >
      <button
        className="tree-folder__header"
        onClick={() => setOpen((v) => !v)}
        style={{ paddingLeft: `${8 + depth * 14}px` }}
      >
        <ChevronRight
          size={11}
          className={`tree-folder__chevron ${open ? "tree-folder__chevron--open" : ""}`}
        />
        {open ? (
          <FolderOpen size={13} className="tree-folder__icon" />
        ) : (
          <Folder size={13} className="tree-folder__icon" />
        )}
        <span className="sidebar__label tree-folder__name truncate">
          {folder.name}
        </span>
      </button>

      {open && (
        <div className="tree-folder__children">
          {visible.map((child) =>
            child.kind === "folder" ? (
              <FolderNode
                key={child.folder.id}
                folder={child.folder}
                depth={depth + 1}
                search={search}
                activeTabId={activeTabId}
                onOpenFile={onOpenFile}
              />
            ) : (
              <FileNode
                key={child.file.id}
                file={child.file}
                depth={depth + 1}
                isActive={activeTabId === child.file.id}
                onClick={() => onOpenFile(child.file)}
              />
            ),
          )}
        </div>
      )}
    </div>
  );
}

function FileNode({
  file,
  depth,
  isActive,
  onClick,
}: {
  file: any;
  depth: number;
  isActive: boolean;
  onClick: () => void;
}) {
  const meta = (FILE_TYPE_META[file.type as FileType] as any) ?? {
    icon: FileText,
    color: "var(--text-muted)",
  };
  const Icon = meta.icon;

  return (
    <button
      className={`tree-file ${isActive ? "tree-file--active" : ""}`}
      style={{ paddingLeft: `${8 + depth * 14}px` }}
      onClick={onClick}
      title={file.name}
    >
      <Icon size={12} style={{ color: meta.color, flexShrink: 0 }} />
      <span className="sidebar__label tree-file__name truncate">
        {file.name}
      </span>
    </button>
  );
}

function PublishButton({ status }: { status: ConnectionStatus }) {
  const icons: Record<ConnectionStatus, React.ReactNode> = {
    connected: <Upload size={13} />,
    connecting: <Loader size={13} className="sidebar__publish-spin" />,
    disconnected: <Unplug size={13} />,
    stagging: <CloudUpload size={13} />,
  };

  const labels: Record<ConnectionStatus, string> = {
    connected: "Published",
    connecting: "Connecting…",
    disconnected: "Disconnected",
    stagging: "Publish",
  };

  return (
    <button className={`sidebar__publish sidebar__publish--${status}`}>
      {icons[status]}
      <span className="sidebar__label">{labels[status]}</span>
      <span className="sidebar__publish-dot" />
    </button>
  );
}
