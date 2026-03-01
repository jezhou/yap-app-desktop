import SearchBar from "../shared/SearchBar";

export default function NavBar() {
  return (
    <header className="flex items-center h-12 px-4 bg-surface-alt border-b border-border">
      <div className="w-72">
        <SearchBar />
      </div>
      <div className="flex-1" />
    </header>
  );
}
