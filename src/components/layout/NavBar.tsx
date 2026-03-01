import { useLocation, Link } from "react-router-dom";
import SearchBar from "../shared/SearchBar";

function Breadcrumbs() {
  const location = useLocation();
  const segments = location.pathname.split("/").filter(Boolean);

  if (segments.length === 0) {
    return <span className="text-sm text-text-muted">Home</span>;
  }

  const crumbs: { label: string; path: string }[] = [
    { label: "Home", path: "/" },
  ];

  if (segments[0] === "settings") {
    crumbs.push({ label: "Settings", path: "/settings" });
  } else if (segments[0] === "setup") {
    crumbs.push({ label: "Setup", path: "/setup" });
  } else {
    // Session path
    crumbs.push({
      label: `Session`,
      path: `/${segments[0]}`,
    });
    if (segments[1]) {
      crumbs.push({
        label: `Conversation`,
        path: `/${segments[0]}/${segments[1]}`,
      });
    }
  }

  return (
    <nav className="flex items-center gap-1 text-sm">
      {crumbs.map((crumb, index) => (
        <span key={crumb.path} className="flex items-center gap-1">
          {index > 0 && <span className="text-text-dim">/</span>}
          {index === crumbs.length - 1 ? (
            <span className="text-text-muted">{crumb.label}</span>
          ) : (
            <Link
              to={crumb.path}
              className="text-text-dim hover:text-text transition-colors"
            >
              {crumb.label}
            </Link>
          )}
        </span>
      ))}
    </nav>
  );
}

export default function NavBar() {
  return (
    <header className="flex items-center h-12 px-4 bg-surface-alt border-b border-border gap-4">
      <Breadcrumbs />
      <div className="flex-1" />
      <div className="w-64">
        <SearchBar />
      </div>
    </header>
  );
}
