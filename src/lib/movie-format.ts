import type { MovieListItem } from "../types/api";

export type ResolutionTier = "4K" | "1080p" | "720p" | "576p" | "480p";

/**
 * Badge tier from stored stream width/height only — not library folder path.
 * UHD rips often report e.g. 3840×1608; 1080p encodes often report 1920×804.
 */
export function resolutionTier(movie: MovieListItem): ResolutionTier | null {
  const width = movie.resolutionWidth ?? 0;
  const height = movie.resolutionHeight ?? 0;

  if (width === 0 && height === 0) {
    return null;
  }

  if (width >= 3840 || height >= 2160) {
    return "4K";
  }

  if (width >= 1920 || height >= 1080) {
    return "1080p";
  }

  if (height >= 720 || width >= 1280) {
    return "720p";
  }

  if (height >= 576) {
    return "576p";
  }

  if (height > 0) {
    return "480p";
  }

  return null;
}

export function effectiveResolutionHeight(movie: MovieListItem): number | null {
  const tier = resolutionTier(movie);
  if (tier === "4K") {
    return movie.resolutionHeight ?? 2160;
  }
  if (tier === "1080p") {
    return movie.resolutionHeight ?? 1080;
  }
  if (tier === "720p") {
    return movie.resolutionHeight ?? 720;
  }
  if (tier === "576p") {
    return 576;
  }
  if (tier === "480p") {
    return movie.resolutionHeight ?? 480;
  }
  return movie.resolutionHeight;
}

export function formatResolution(movie: MovieListItem) {
  if (movie.resolutionWidth && movie.resolutionHeight) {
    return `${movie.resolutionWidth}×${movie.resolutionHeight}`;
  }
  const h = movie.resolutionHeight;
  const w = movie.resolutionWidth;
  if (w && h) {
    return `${w}×${h}`;
  }
  if (h) {
    return `—×${h}`;
  }
  return "—";
}

export function formatResolutionShort(movie: MovieListItem) {
  return resolutionTier(movie) ?? "—";
}

export function formatFileSize(bytes: number | null) {
  if (bytes == null || bytes <= 0) {
    return "—";
  }
  const units = ["B", "KB", "MB", "GB", "TB"] as const;
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

export function absolutePathFromMovie(movie: MovieListItem) {
  if (!movie.driveLetter) {
    return null;
  }
  return `${movie.driveLetter}${movie.relativePath}`;
}