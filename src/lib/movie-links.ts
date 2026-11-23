import { revealItemInDir } from "@tauri-apps/plugin-opener";
import type { MovieListItem } from "../types/api";
import { absolutePathFromMovie } from "./movie-format";

export function imdbPageUrl(rawId: string) {
  const id = rawId.trim();
  const titleId = id.startsWith("tt") ? id : `tt${id}`;
  return `https://www.imdb.com/title/${titleId}/`;
}

export function tmdbPageUrl(rawId: string) {
  const numeric = rawId.trim().replace(/^tmdb:/i, "");
  return `https://www.themoviedb.org/movie/${numeric}`;
}

export async function revealMovieDirectory(movie: MovieListItem) {
  const path = absolutePathFromMovie(movie);
  if (!path) {
    throw new Error("Drive offline — folder path is unavailable.");
  }
  await revealItemInDir(path);
}