import { useState, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { searchConversations } from "../../services/search";
import type { SearchResult } from "../../types";

export default function SearchBar() {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [showDropdown, setShowDropdown] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    if (!query.trim()) {
      setResults([]);
      setShowDropdown(false);
      return;
    }

    debounceRef.current = setTimeout(async () => {
      try {
        const searchResults = await searchConversations(query, 10);
        setResults(searchResults);
        setShowDropdown(searchResults.length > 0);
      } catch {
        setResults([]);
      }
    }, 300);

    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, [query]);

  // Close dropdown on outside click
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        setShowDropdown(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  function handleSelect(result: SearchResult) {
    navigate(`/${result.session_id}/${result.conversation_id}`);
    setQuery("");
    setShowDropdown(false);
  }

  return (
    <div ref={containerRef} className="relative">
      <input
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onFocus={() => {
          if (results.length > 0) setShowDropdown(true);
        }}
        placeholder="Search conversations..."
        className="w-full bg-bg border border-border rounded-lg px-3 py-2 text-sm text-text placeholder-text-dim focus:outline-none focus:border-accent transition-colors"
      />

      {showDropdown && (
        <div className="absolute z-50 top-full mt-1 w-full bg-surface border border-border rounded-lg shadow-lg overflow-hidden">
          {results.map((result, index) => (
            <button
              key={`${result.conversation_id}-${index}`}
              onClick={() => handleSelect(result)}
              className="w-full text-left px-3 py-2.5 hover:bg-surface-hover transition-colors border-b border-border last:border-b-0"
            >
              <p className="text-sm font-medium text-text truncate">
                {result.title}
              </p>
              <p className="text-xs text-text-muted mt-0.5 truncate">
                {result.snippet}
              </p>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
