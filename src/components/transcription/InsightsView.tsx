interface InsightsViewProps {
  content: string;
}

export default function InsightsView({ content }: InsightsViewProps) {
  if (!content) {
    return (
      <p className="text-text-muted text-center py-8">
        No insights available.
      </p>
    );
  }

  // Split content into lines; first line is the summary sentence,
  // remaining lines starting with "- " are key insights
  const lines = content.split("\n").filter((line) => line.trim());
  const summary = lines[0] ?? "";
  const insights = lines.slice(1).filter((line) => line.startsWith("- "));

  return (
    <div className="space-y-4">
      {summary && (
        <div className="bg-surface rounded-lg p-4 border border-border">
          <h3 className="text-xs font-medium text-text-muted uppercase tracking-wider mb-2">
            Summary
          </h3>
          <p className="text-text">{summary}</p>
        </div>
      )}

      {insights.length > 0 && (
        <div className="bg-surface rounded-lg p-4 border border-border">
          <h3 className="text-xs font-medium text-text-muted uppercase tracking-wider mb-2">
            Key Insights
          </h3>
          <ul className="space-y-2">
            {insights.map((insight, index) => (
              <li key={index} className="flex gap-2 text-sm text-text">
                <span className="text-accent flex-shrink-0">*</span>
                <span>{insight.replace(/^-\s*/, "")}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
