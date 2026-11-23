import { useEffect, useMemo, useState } from "react";
import { fetchMoviePage } from "../../lib/library-query";
import { imdbPageUrl, revealMovieDirectory, tmdbPageUrl } from "../../lib/movie-links";
import { formatFileSize, formatResolutionShort } from "../../lib/movie-format";
import { openUrl } from "@tauri-apps/plugin-opener";
import { cn } from "../../lib/cn";
import type { MovieListItem } from "../../types/api";
import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { StatusPill } from "../ui/StatusPill";

type Props = {
  refreshToken: number;
  busy?: boolean;
  onDeleteCopy: (movie: MovieListItem) => void;
  onStatus: (message: string) => void;
};

export function DuplicatesView({
  refreshToken,
  busy = false,
  onDeleteCopy,
  onStatus,
}: Props) {
  const [movies, setMovies] = useState<MovieListItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      setLoading(true);
      try {
        const all: MovieListItem[] = [];
        let offset = 0;
        let more = true;
        while (more) {
          const page = await fetchMoviePage("duplicates", "all", "", offset);
          all.push(...page.items);
          more = page.hasMore;
          offset += page.items.length;
          if (offset > 0 && page.items.length === 0) {
            break;
          }
        }
        setMovies(all);
      } catch (err) {
        onStatus(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    }
    void load();
  }, [refreshToken, onStatus]);

  const groups = useMemo(() => {
    const map = new Map<number, MovieListItem[]>();
    for (const movie of movies) {
      if (movie.duplicateGroupId == null) {
        continue;
      }
      const list = map.get(movie.duplicateGroupId) ?? [];
      list.push(movie);
      map.set(movie.duplicateGroupId, list);
    }
    return [...map.entries()].sort((a, b) =>
      a[1][0].title.localeCompare(b[1][0].title),
    );
  }, [movies]);

  const [selectedId, setSelectedId] = useState<number | null>(null);

  useEffect(() => {
    setSelectedId((current) => {
      if (groups.length === 0) {
        return null;
      }
      if (current != null && groups.some(([id]) => id === current)) {
        return current;
      }
      return groups[0][0];
    });
  }, [groups]);

  const selected = groups.find(([id]) => id === selectedId) ?? null;

  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center rounded-xl border border-cv-border bg-cv-surface p-8 text-sm text-cv-muted">
        Loading duplicate groups…
      </div>
    );
  }

  if (groups.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center rounded-xl border border-cv-border bg-cv-surface p-8 text-center">
        <p className="text-pretty max-w-sm text-sm text-cv-muted">
          No duplicate groups yet. Scan folders that share an IMDb id or the same title and year.
        </p>
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 gap-3">
      <ul className="w-72 shrink-0 overflow-auto rounded-xl border border-cv-border bg-cv-surface">
        {groups.map(([id, copies]) => (
          <li key={id} className="border-b border-cv-border last:border-b-0">
            <button
              type="button"
              onClick={() => setSelectedId(id)}
              className={cn(
                "flex w-full flex-col items-start gap-0.5 px-3 py-2.5 text-left hover:bg-cv-surface-raised",
                selected?.[0] === id && "bg-cv-surface-raised",
              )}
            >
              <span className="truncate text-sm font-medium">{copies[0].title}</span>
              <span className="tabular-nums text-xs text-cv-muted">
                {copies[0].year ?? "—"} · {copies.length} copies
              </span>
            </button>
          </li>
        ))}
      </ul>
      <div className="grid min-w-0 flex-1 content-start gap-3 overflow-auto md:grid-cols-2">
        {selected?.[1].map((movie) => (
          <article
            key={movie.id}
            className="rounded-xl border border-cv-border bg-cv-surface p-4"
          >
            <h2 className="text-balance text-sm font-semibold">{movie.title}</h2>
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <Badge>{formatResolutionShort(movie)}</Badge>
              <Badge mono>{movie.driveLetter ?? "—"}</Badge>
              <StatusPill status={movie.status} />
            </div>
            {(selected?.[1].length ?? 0) > 1 && movie.status !== "offline" ? (
              <Button
                className="mt-4"
                variant="danger"
                size="sm"
                disabled={busy}
                onClick={() => onDeleteCopy(movie)}
              >
                Delete this copy
              </Button>
            ) : null}
            <dl className="mt-4 space-y-2 text-sm">
              {movie.imdbId ? (
                <div className="grid grid-cols-[72px_minmax(0,1fr)] gap-2">
                  <dt className="text-cv-muted">IMDb</dt>
                  <dd>
                    <button
                      type="button"
                      className="text-left text-cv-accent hover:underline"
                      onClick={() => {
                        void openUrl(imdbPageUrl(movie.imdbId!)).catch((err) =>
                          onStatus(err instanceof Error ? err.message : String(err)),
                        );
                      }}
                    >
                      {movie.imdbId}
                    </button>
                  </dd>
                </div>
              ) : null}
              {movie.tmdbId ? (
                <div className="grid grid-cols-[72px_minmax(0,1fr)] gap-2">
                  <dt className="text-cv-muted">TMDb</dt>
                  <dd>
                    <button
                      type="button"
                      className="text-left text-cv-accent hover:underline"
                      onClick={() => {
                        void openUrl(tmdbPageUrl(movie.tmdbId!)).catch((err) =>
                          onStatus(err instanceof Error ? err.message : String(err)),
                        );
                      }}
                    >
                      {movie.tmdbId}
                    </button>
                  </dd>
                </div>
              ) : null}
              <div className="grid grid-cols-[72px_minmax(0,1fr)] gap-2">
                <dt className="text-cv-muted">Size</dt>
                <dd className="tabular-nums">{formatFileSize(movie.fileSize)}</dd>
              </div>
              <div className="grid grid-cols-[72px_minmax(0,1fr)] gap-2">
                <dt className="text-cv-muted">Path</dt>
                <dd className="break-all text-xs">
                  {movie.status !== "offline" && movie.driveLetter ? (
                    <button
                      type="button"
                      className="text-left text-cv-accent hover:underline"
                      onClick={() => {
                        void revealMovieDirectory(movie).catch((err) =>
                          onStatus(err instanceof Error ? err.message : String(err)),
                        );
                      }}
                    >
                      {movie.driveLetter}
                      {movie.relativePath}
                    </button>
                  ) : (
                    <span className="text-cv-muted">
                      {movie.driveLetter
                        ? `${movie.driveLetter}${movie.relativePath}`
                        : movie.relativePath}
                    </span>
                  )}
                </dd>
              </div>
            </dl>
          </article>
        ))}
      </div>
    </div>
  );
}