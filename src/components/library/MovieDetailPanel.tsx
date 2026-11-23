import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { X } from "lucide-react";
import { cn } from "../../lib/cn";
import { imdbPageUrl, tmdbPageUrl } from "../../lib/movie-links";
import {
  absolutePathFromMovie,
  formatFileSize,
  formatResolution,
} from "../../lib/movie-format";
import type { MovieListItem } from "../../types/api";
import { IconButton } from "../ui/IconButton";

type Props = {
  movie: MovieListItem;
  onClose: () => void;
  onStatus?: (message: string) => void;
};

function MetaRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[88px_minmax(0,1fr)] gap-2 text-sm">
      <dt className="text-cv-muted">{label}</dt>
      <dd className="min-w-0 break-all">{value}</dd>
    </div>
  );
}

function MetaExternalLink({
  label,
  display,
  href,
  onStatus,
}: {
  label: string;
  display: string;
  href: string;
  onStatus?: (message: string) => void;
}) {
  return (
    <div className="grid grid-cols-[88px_minmax(0,1fr)] gap-2 text-sm">
      <dt className="text-cv-muted">{label}</dt>
      <dd className="min-w-0">
        <button
          type="button"
          onClick={() => {
            void openUrl(href).catch((err) =>
              onStatus?.(err instanceof Error ? err.message : String(err)),
            );
          }}
          className={cn(
            "max-w-full break-all text-left text-cv-accent hover:underline",
            "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-cv-accent",
          )}
          title={href}
        >
          {display}
        </button>
      </dd>
    </div>
  );
}

function MetaFolderLink({
  label,
  path,
  enabled,
  onStatus,
}: {
  label: string;
  path: string;
  enabled: boolean;
  onStatus?: (message: string) => void;
}) {
  return (
    <div className="grid grid-cols-[88px_minmax(0,1fr)] gap-2 text-sm">
      <dt className="text-cv-muted">{label}</dt>
      <dd className="min-w-0">
        {enabled ? (
          <button
            type="button"
            onClick={() => {
              void revealItemInDir(path).catch((err) =>
                onStatus?.(err instanceof Error ? err.message : String(err)),
              );
            }}
            className={cn(
              "max-w-full break-all text-left text-cv-accent hover:underline",
              "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-cv-accent",
            )}
            title="Show in File Explorer"
          >
            {path}
          </button>
        ) : (
          <span className="break-all text-cv-muted" title="Drive offline">
            {path}
          </span>
        )}
      </dd>
    </div>
  );
}

export function MovieDetailPanel({ movie, onClose, onStatus }: Props) {
  const path = absolutePathFromMovie(movie) ?? movie.relativePath;
  const folderOpenable = movie.status !== "offline" && Boolean(absolutePathFromMovie(movie));

  return (
    <aside
      className="flex w-80 shrink-0 flex-col overflow-hidden rounded-xl border border-cv-border bg-cv-surface"
      aria-label="Movie details"
    >
      <div className="flex items-start justify-between gap-2 border-b border-cv-border px-3 py-2">
        <h2 className="text-balance pt-1 text-sm font-semibold">{movie.title}</h2>
        <IconButton label="Close details" icon={<X className="size-4" />} onClick={onClose} />
      </div>
      <dl className="space-y-3 overflow-auto p-4">
        <MetaRow label="Year" value={movie.year?.toString() ?? "—"} />
        <MetaRow label="Resolution" value={formatResolution(movie)} />
        <MetaRow label="Codec" value={movie.videoCodec ?? "—"} />
        <MetaRow label="HDR" value={movie.hdr ? "Yes" : "No"} />
        <MetaRow label="Size" value={formatFileSize(movie.fileSize)} />
        <MetaRow label="Drive" value={movie.driveLetter ?? "—"} />
        <MetaRow label="Status" value={movie.status} />
        {movie.imdbId ? (
          <MetaExternalLink
            label="IMDb"
            display={movie.imdbId}
            href={imdbPageUrl(movie.imdbId)}
            onStatus={onStatus}
          />
        ) : (
          <MetaRow label="IMDb" value="—" />
        )}
        {movie.tmdbId ? (
          <MetaExternalLink
            label="TMDb"
            display={movie.tmdbId}
            href={tmdbPageUrl(movie.tmdbId)}
            onStatus={onStatus}
          />
        ) : (
          <MetaRow label="TMDb" value="—" />
        )}
        <MetaRow
          label="Duplicate"
          value={movie.duplicateGroupId != null ? `Group ${movie.duplicateGroupId}` : "—"}
        />
        <MetaFolderLink
          label="Path"
          path={path}
          enabled={folderOpenable}
          onStatus={onStatus}
        />
      </dl>
    </aside>
  );
}