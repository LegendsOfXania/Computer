import { useRef } from "react";
import { X, GitBranch, Home } from "lucide-react";
import { FILE_TYPE_META } from "../../types/fileTypes";
import { useApp } from "../../store/AppContext";
import type { Tab, FileType } from "../../types";
import "./TabBar.css";

export function TabBar() {
  const { tabs, activeTabId, setActiveTab, closeTab, allFiles } = useApp();
  const scrollRef = useRef<HTMLDivElement>(null);

  const getFileType = (tab: Tab): FileType | null => {
    if (tab.kind !== "file" || !tab.fileId) return null;
    return allFiles.find((f) => f.id === tab.fileId)?.type ?? null;
  };

  return (
    <div className="tabbar">
      <div className="tabbar__scroll" ref={scrollRef}>
        {tabs.map((tab) => {
          const fileType = getFileType(tab);
          const Icon =
            tab.kind === "home"
              ? Home
              : fileType
                ? FILE_TYPE_META[fileType].icon
                : GitBranch;
          const iconColor = fileType
            ? FILE_TYPE_META[fileType].color
            : "var(--text-muted)";
          const isActive = tab.id === activeTabId;

          return (
            <button
              key={tab.id}
              className={`tabbar__tab ${isActive ? "tabbar__tab--active" : ""}`}
              onClick={() => setActiveTab(tab.id)}
              style={{ "--tab-accent": iconColor } as React.CSSProperties}
            >
              <Icon size={12} style={{ color: iconColor }} />
              <span className="tabbar__tab-label truncate">{tab.label}</span>
              {tab.dirty && (
                <span className="tabbar__tab-dirty" title="Unsaved changes" />
              )}
              {tab.kind !== "home" && (
                <span
                  className="tabbar__tab-close"
                  role="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    closeTab(tab.id);
                  }}
                  title="Close"
                >
                  <X size={11} />
                </span>
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
}
