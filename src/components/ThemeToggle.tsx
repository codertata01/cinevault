import { useEffect, useState } from "react";
import {
  applyTheme,
  getStoredTheme,
  oppositeTheme,
  type Theme,
} from "../lib/theme";
import { cn } from "../lib/cn";

export function ThemeToggle() {
  const [theme, setTheme] = useState<Theme>(() => getStoredTheme());

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  const next = oppositeTheme(theme);

  return (
    <button
      type="button"
      aria-label={`Switch to ${next} theme`}
      aria-pressed={theme === "dark"}
      onClick={() => setTheme(next)}
      className={cn(
        "rounded-md border border-cv-border px-3 py-1.5 text-sm text-cv-muted",
        "hover:bg-cv-surface-raised hover:text-cv-foreground",
      )}
    >
      {theme === "dark" ? "Light" : "Dark"}
    </button>
  );
}