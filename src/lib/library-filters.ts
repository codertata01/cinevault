import type { MovieListItem } from "../types/api";

export type LibraryKindFilter = "all" | "duplicates" | "4k" | "1080p" | "offline";
export type LibraryLayoutMode = "table" | "grid";

export function driveLettersFromMovies(movies: MovieListItem[]) {
  const set = new Set<string>();
  for (const movie of movies) {
    if (movie.driveLetter) {
      set.add(movie.driveLetter);
    }
  }
  return [...set].sort();
}

export function filterMovies(
  movies: MovieListItem[],
  kind: LibraryKindFilter,
  drive: string | "all",
) {
  let list = movies;
  if (drive !== "all") {
    list = list.filter((movie) => movie.driveLetter === drive);
  }

  switch (kind) {
    case "duplicates":
      return list.filter((movie) => movie.duplicateGroupId != null);
    case "4k":
      return list.filter(
        (movie) =>
          (movie.resolutionWidth ?? 0) >= 3840 || (movie.resolutionHeight ?? 0) >= 2160,
      );
    case "1080p":
      return list.filter((movie) => {
        const w = movie.resolutionWidth ?? 0;
        const h = movie.resolutionHeight ?? 0;
        const is4k = w >= 3840 || h >= 2160;
        return (
          !is4k &&
          ((w >= 1920 && w < 3840) || (h >= 1080 && h < 2160))
        );
      });
    case "offline":
      return list.filter((movie) => movie.status === "offline");
    default:
      return list;
  }
}

export function readStoredLayout(): LibraryLayoutMode {
  const value = localStorage.getItem("cinevault-library-layout");
  if (value === "table" || value === "grid") {
    return value;
  }
  if (value === "list") {
    return "table";
  }
  return "table";
}

export function storeLayout(mode: LibraryLayoutMode) {
  localStorage.setItem("cinevault-library-layout", mode);
}

const FILTER_STORAGE_KEY = "cinevault-library-filters";

export type StoredLibraryFilters = {
  kind: LibraryKindFilter;
  drive: string | "all";
  search: string;
};

export function readStoredFilters(): Partial<StoredLibraryFilters> {
  try {
    const raw = sessionStorage.getItem(FILTER_STORAGE_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw) as Partial<StoredLibraryFilters>;
    const kind = parsed.kind;
    const validKind =
      kind === "all" ||
      kind === "duplicates" ||
      kind === "4k" ||
      kind === "1080p" ||
      kind === "offline"
        ? kind
        : undefined;
    const drive =
      parsed.drive === "all"
        ? "all"
        : typeof parsed.drive === "string" && /^[A-Z]:$/i.test(parsed.drive)
          ? parsed.drive.toUpperCase()
          : undefined;
    const search = typeof parsed.search === "string" ? parsed.search : undefined;
    return { kind: validKind, drive, search };
  } catch {
    return {};
  }
}

export function storeFilters(filters: StoredLibraryFilters) {
  sessionStorage.setItem(FILTER_STORAGE_KEY, JSON.stringify(filters));
}