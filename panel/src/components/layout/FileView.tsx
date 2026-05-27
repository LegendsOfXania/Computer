import { useApp } from "../../store/AppContext";
import { FILE_TYPE } from "../../types/fileTypes";
import "./FileView.css";

export function fileView({ fileId }: { fileId: string }) {
  const { allFiles } = useApp();
  const file = allFiles.find((f) => f.id === fileId);

  if (!file) {
    return (
      <div className="file-view file-view--not-found">
        <span>File not found</span>
      </div>
    );
  }

  const meta = FILE_TYPE_META[file.type];
  const Icon = meta.icon;

  return (
    <div className="file-view">
      <div
        className="file-view__header"
        style={{ "--file-color": meta.color } as React.CSSProperties}
      >
        <div className="file-view__header-left">
          <Icon size={14} style={{ color: meta.color }} />
          <span className="file-view__name mono">{file.name}</span>
          <span className="file-view__badge" style={{ color: meta.color }}>
            {meta.label}
          </span>
        </div>
        <div className="file-view__header-right">
          <span className="file-view__meta file-view__id">{file.id}</span>
          <span className="file-view__meta">Priority: {file.priority}</span>
          <span className="file-view__meta">|</span>
          <span className="file-view__meta">
            {file.entries.length} entries
          </span>
        </div>
      </div>

      <div className="file-view__placeholder">
        <div className="file-view__placeholder-inner">
          <Icon size={28} style={{ color: meta.color, opacity: 0.3 }} />
          <p className="file-view__placeholder-label">{meta.editorLabel}</p>
          <p className="file-view__placeholder-hint">
            Editor for <strong>{file.type}</strong> files — coming next
          </p>
        </div>
      </div>
    </div>
  );
}
