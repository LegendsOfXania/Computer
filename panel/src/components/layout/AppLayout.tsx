import { useApp } from "../../store/AppContext";
import { Sidebar } from "../sidebar/Sidebar";
import { TabBar } from "../tabs/TabBar";
import { HomeView } from "./HomeView";
import { FileView } from "./FileView";
import "./AppLayout.css";

export function AppLayout() {
  const { tabs, activeTabId } = useApp();
  const activeTab = tabs.find((t) => t.id === activeTabId);

  return (
    <div className="app-layout">
      <Sidebar />
      <div className="app-layout__main">
        <TabBar />
        <div className="app-layout__content">
          {activeTab?.kind === "home" && <HomeView />}
          {activeTab?.kind === "file" && activeTab.fileId && (
            <FileView fileId={activeTab.fileId} />
          )}
          {!activeTab && (
            <div className="app-layout__empty">
              <span>No file open</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
