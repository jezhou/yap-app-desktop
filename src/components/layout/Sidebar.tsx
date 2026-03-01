import { useState } from "react";
import { NavLink } from "react-router-dom";

export default function Sidebar() {
  const [collapsed, setCollapsed] = useState(false);

  return (
    <aside
      className={`flex flex-col bg-surface border-r border-border transition-all duration-200 ${
        collapsed ? "w-16" : "w-64"
      }`}
    >
      <div className="flex items-center justify-between p-4 border-b border-border">
        {!collapsed && (
          <span className="text-lg font-bold text-accent">Yap</span>
        )}
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="p-1.5 rounded hover:bg-surface-hover text-text-muted hover:text-text transition-colors"
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {collapsed ? "\u25B6" : "\u25C0"}
        </button>
      </div>

      <nav className="flex-1 overflow-y-auto p-2">
        <NavLink
          to="/"
          end
          className={({ isActive }) =>
            `flex items-center gap-3 px-3 py-2 rounded text-sm transition-colors ${
              isActive
                ? "bg-accent/10 text-accent"
                : "text-text-muted hover:bg-surface-hover hover:text-text"
            }`
          }
        >
          {!collapsed && <span>Sessions</span>}
        </NavLink>
        <NavLink
          to="/settings"
          className={({ isActive }) =>
            `flex items-center gap-3 px-3 py-2 rounded text-sm transition-colors ${
              isActive
                ? "bg-accent/10 text-accent"
                : "text-text-muted hover:bg-surface-hover hover:text-text"
            }`
          }
        >
          {!collapsed && <span>Settings</span>}
        </NavLink>
      </nav>
    </aside>
  );
}
