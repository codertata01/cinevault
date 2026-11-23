import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useRef, useState } from "react";
import { cn } from "../../lib/cn";
import { useInfiniteLoadNearEnd } from "../../lib/use-infinite-load";
import { formatResolutionShort } from "../../lib/movie-format";
import type { MovieListItem } from "../../types/api";
import { Badge } from "../ui/Badge";
import { PosterFrame } from "./PosterFrame";

const COLUMN_WIDTH = 156;
const ROW_HEIGHT = 292;

type Props = {
  movies: MovieListItem[];
  selectedId: number | null;
  onSelect: (movie: MovieListItem) => void;
  hasMore?: boolean;
  loading?: boolean;
  onLoadMore?: () => void;
};

export function MovieGridView({
  movies,
  selectedId,
  onSelect,
  hasMore = false,
  loading = false,
  onLoadMore,
}: Props) {
  const parentRef = useRef<HTMLDivElement>(null);
  const [columns, setColumns] = useState(4);
  const rowCount = Math.ceil(movies.length / columns);

  useEffect(() => {
    const element = parentRef.current;
    if (!element) {
      return;
    }
    const update = () => {
      setColumns(Math.max(1, Math.floor(element.clientWidth / COLUMN_WIDTH)));
    };
    update();
    const observer = new ResizeObserver(update);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const virtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 4,
  });

  const virtualRows = virtualizer.getVirtualItems();
  const lastVisibleRow =
    virtualRows.length > 0 ? virtualRows[virtualRows.length - 1].index : 0;
  useInfiniteLoadNearEnd({
    enabled: Boolean(onLoadMore) && hasMore,
    loading,
    lastVisibleIndex: lastVisibleRow,
    totalCount: rowCount,
    threshold: 3,
    onLoadMore: onLoadMore ?? (() => {}),
  });

  return (
    <div ref={parentRef} className="h-full overflow-auto">
      <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
        {virtualRows.map((virtualRow) => {
          const start = virtualRow.index * columns;
          const rowMovies = movies.slice(start, start + columns);
          return (
            <div
              key={virtualRow.key}
              className="absolute left-0 top-0 grid w-full gap-3 px-3"
              style={{
                transform: `translateY(${virtualRow.start}px)`,
                height: ROW_HEIGHT,
                gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
              }}
            >
              {rowMovies.map((movie) => (
                <button
                  key={movie.id}
                  type="button"
                  onClick={() => onSelect(movie)}
                  className={cn(
                    "flex h-[280px] flex-col overflow-hidden rounded-lg border border-cv-border bg-cv-bg text-left hover:border-cv-accent-muted",
                    selectedId === movie.id && "border-cv-accent",
                  )}
                >
                  <PosterFrame movie={movie} />
                  <div className="space-y-1 p-2">
                    <p className="line-clamp-2 text-xs font-medium leading-snug">{movie.title}</p>
                    <p className="flex items-center gap-1.5 text-[11px] text-cv-muted">
                      <span className="tabular-nums">{movie.year ?? "—"}</span>
                      <Badge>{formatResolutionShort(movie)}</Badge>
                      <Badge mono>{movie.driveLetter ?? "—"}</Badge>
                    </p>
                  </div>
                </button>
              ))}
            </div>
          );
        })}
      </div>
    </div>
  );
}