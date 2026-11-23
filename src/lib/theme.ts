export type Theme = "dark" | "light";

const STORAGE_KEY = "cinevault-theme";

export function getStoredTheme(): Theme {
  const value = localStorage.getItem(STORAGE_KEY);
  if (value === "light" || value === "dark") {
    return value;
  }
  return "dark";
}

export function applyTheme(theme: Theme) {
  document.documentElement.dataset.theme = theme;
  localStorage.setItem(STORAGE_KEY, theme);
}

export function initTheme() {
  applyTheme(getStoredTheme());
}

export function oppositeTheme(theme: Theme): Theme {
  return theme === "dark" ? "light" : "dark";
}