import { useApp } from "../../store/AppContext";
import { FILE_TYPE } from "../../types/fileTypes";
import type { FileType } from "../../types";
import "./HomeView.css";

export function HomeView() {
  const { allFiles, openTab } = useApp();

  const counts = (Object.keys(FILE_TYPE) as FileType[]).map((type) => ({
    type,
    count: allFiles.filter((f) => f.type === type).length,
  }));

  return (
    <div className="home">
      <div className="home__hero">
        <div className="home__hero-glow" />
        <h1 className="home__title">Computer</h1>
        <p className="home__subtitle">
          PumpkinMC — Player interactions engine panel
        </p>
      </div>

      <div className="home__stats">
        {counts.map(({ type, count }) => {
          const { label, icon: Icon, color, desc } = FILE_TYPE[type];
          return (
            <div
              key={type}
              className="home__stat"
              style={{ "--stat-color": color } as React.CSSProperties}
            >
              <Icon size={20} style={{ color }} />
              <div className="home__stat-info">
                <span className="home__stat-count">{count}</span>
                <span className="home__stat-label">{label}</span>
              </div>
              <p className="home__stat-desc">{desc}</p>
            </div>
          );
        })}
      </div>

      <div className="home__recent">
        <h2 className="home__section-title">Recent files</h2>
        <div className="home__file-list">
          {allFiles.slice(0, 6).map((file) => {
            const { icon: Icon, color } = FILE_TYPE[file.type];
            return (
              <button
                key={file.id}
                className="home__file-item"
                onClick={() =>
                  openTab({
                    id: file.id,
                    kind: "file",
                    label: file.name,
                    fileId: file.id,
                  })
                }
              >
                <Icon size={13} style={{ color }} />
                <span className="home__file-name mono">{file.name}</span>
                <span className="home__file-type" style={{ color }}>
                  {file.type}
                </span>
                <span className="home__file-id">{file.id}</span>
                <span className="home__file-entries">
                  {file.entries.length} entries
                </span>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
