import { convertFileSrc } from "@tauri-apps/api/core";
import { useState } from "react";
import type { MovieListItem } from "../../types/api";

function initial(title: string) {
  const match = title.trim().match(/[a-z0-9]/i);
  return match ? match[0].toUpperCase() : "?";
}

export function PosterFrame({
  movie,
  compact = false,
}: {
  movie: MovieListItem;
  compact?: boolean;
}) {
  const [missing, setMissing] = useState(false);
  const online = movie.status !== "offline" && movie.driveLetter;
  const src =
    online && !missing
      ? convertFileSrc(`${movie.driveLetter}${movie.relativePath}\\poster.jpg`)
      : null;

  return (
    <div
      className={
        compact
          ? "flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-md bg-cv-surface-raised"
          : "relative flex aspect-[2/3] items-center justify-center overflow-hidden bg-cv-surface-raised"
      }
    >
      {src ? (
        <img
          src={src}
          alt=""
          className="size-full object-cover"
          onError={() => setMissing(true)}
        />
      ) : (
        <span className={compact ? "text-xs font-semibold text-cv-muted" : "text-3xl font-semibold text-cv-muted"}>
          {initial(movie.title)}
        </span>
      )}
    </div>
  );
}