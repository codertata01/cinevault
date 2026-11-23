import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import { cn } from "../../lib/cn";
import { useInfiniteLoadNearEnd } from "../../lib/use-infinite-load";
import { formatResolutionShort } from "../../lib/movie-format";
import type { MovieListItem } from "../../types/api";
import { StatusPill } from "../ui/StatusPill";

const columns = ["Title", "Year", "Resolution", "Drive", "Status"];
const ROW_HEIGHT = 40;

type Props = {
  movies: MovieListItem[];
  selectedId: number | null;
  onSelect: (movie: MovieListItem) => void;
  hasMore?: boolean;
  loading?: boolean;
  onLoadMore?: () => void;
};

export function MovieTableView({
  movies,
  selectedId,
  onSelect,
  hasMore = false,
  loading = false,
  onLoadMore,
}: Props) {
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: movies.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 16,
  });

  const virtualRows = virtualizer.getVirtualItems();
  const lastVisibleIndex =
    virtualRows.length > 0 ? virtualRows[virtualRows.length - 1].index : 0;
  useInfiniteLoadNearEnd({
    enabled: Boolean(onLoadMore) && hasMore,
    loading,
    lastVisibleIndex,
    totalCount: movies.length,
    onLoadMore: onLoadMore ?? (() => {}),
  });

  return (
    <div ref={parentRef} className="h-full overflow-auto">
      <div className="sticky top-0 z-10 grid grid-cols-[minmax(0,2fr)_0.5fr_0.7fr_0.6fr_0.8fr] gap-3 border-b border-cv-border bg-cv-surface px-4 py-2 text-xs font-medium text-cv-muted">
        {columns.map((column) => (
          <span key={column}>{column}</span>
        ))}
      </div>
      <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
        {virtualRows.map((virtualRow) => {
          const movie = movies[virtualRow.index];
          return (
            <div
              key={movie.id}
              className="absolute left-0 top-0 w-full"
              style={{ transform: `translateY(${virtualRow.start}px)` }}
            >
              <button
                type="button"
                onClick={() => onSelect(movie)}
                className={cn(
                  "grid h-10 w-full grid-cols-[minmax(0,2fr)_0.5fr_0.7fr_0.6fr_0.8fr] items-center justify-items-start gap-3 border-b border-cv-border px-4 text-left text-sm hover:bg-cv-surface-raised",
                  selectedId === movie.id && "bg-cv-surface-raised",
                )}
              >
                <span className="min-w-0 w-full truncate font-medium">{movie.title}</span>
                <span className="tabular-nums text-cv-muted">{movie.year ?? "—"}</span>
                <span className="text-xs tabular-nums">{formatResolutionShort(movie)}</span>
                <span className="font-mono text-xs tabular-nums text-cv-muted">
                  {movie.driveLetter ?? "—"}
                </span>
                <StatusPill status={movie.status} />
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}