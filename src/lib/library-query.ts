import { invoke } from "@tauri-apps/api/core";
import type { MovieListItem } from "../types/api";
import type { LibraryKindFilter } from "./library-filters";

export const MOVIE_PAGE_SIZE = 500;

export type MovieQueryResult = {
  items: MovieListItem[];
  total: number;
  offset: number;
  limit: number;
  hasMore: boolean;
};

export type DriveLetterCount = {
  driveLetter: string;
  count: number;
};

export async function fetchMoviePage(
  kind: LibraryKindFilter,
  drive: string | "all",
  search: string,
  offset: number,
) {
  return invoke<MovieQueryResult>("query_movies", {
    query: {
      search: search.trim() || null,
      kind: kind === "all" ? null : kind,
      driveLetter: drive === "all" ? null : drive,
      limit: MOVIE_PAGE_SIZE,
      offset,
    },
  });
}

export async function fetchDriveCounts() {
  return invoke<DriveLetterCount[]>("list_drive_letter_counts");
}