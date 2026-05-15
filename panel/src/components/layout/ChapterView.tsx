import { useApp } from "../../store/AppContext";
import { FILE_TYPE_META } from "../../types/fileTypes";
import "./ChapterView.css";

export function ChapterView({ fileId }: { fileId: string }) {
  const { allFiles } = useApp();
  const file = allFiles.find((f) => f.id === fileId);

  if (!file) {
    return (
      <div className="chapter-view chapter-view--not-found">
        <span>File not found</span>
      </div>
    );
  }

  const meta = FILE_TYPE_META[file.type];
  const Icon = meta.icon;

  return (
    <div className="chapter-view">
      <div
        className="chapter-view__header"
        style={{ "--chapter-color": meta.color } as React.CSSProperties}
      >
        <div className="chapter-view__header-left">
          <Icon size={14} style={{ color: meta.color }} />
          <span className="chapter-view__name mono">{file.name}</span>
          <span className="chapter-view__badge" style={{ color: meta.color }}>
            {meta.label}
          </span>
        </div>
        <div className="chapter-view__header-right">
          <span className="chapter-view__meta chapter-view__id">{file.id}</span>
          <span className="chapter-view__meta">Priority: {file.priority}</span>
          <span className="chapter-view__meta">|</span>
          <span className="chapter-view__meta">
            {file.entries.length} entries
          </span>
        </div>
      </div>

      <div className="chapter-view__placeholder">
        <div className="chapter-view__placeholder-inner">
          <Icon size={28} style={{ color: meta.color, opacity: 0.3 }} />
          <p className="chapter-view__placeholder-label">{meta.editorLabel}</p>
          <p className="chapter-view__placeholder-hint">
            Editor for <strong>{file.type}</strong> files — coming next
          </p>
        </div>
      </div>
    </div>
  );
}
