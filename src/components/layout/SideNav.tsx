import { Clapperboard, Copy, Moon, Sun } from "lucide-react";
import { cn } from "../../lib/cn";
import type { Theme } from "../../lib/theme";

type View = "library" | "duplicates";

type Props = {
  view: View;
  duplicateCount: number;
  theme: Theme;
  onViewChange: (view: View) => void;
  onToggleTheme: () => void;
};

export function SideNav({
  view,
  duplicateCount,
  theme,
  onViewChange,
  onToggleTheme,
}: Props) {
  const items = [
    { id: "library" as const, label: "Library", icon: Clapperboard },
    { id: "duplicates" as const, label: "Duplicates", icon: Copy },
  ];

  return (
    <nav
      aria-label="Primary"
      className="flex w-14 shrink-0 flex-col border-r border-cv-border bg-cv-surface md:w-52"
    >
      <div className="flex h-14 items-center px-3 md:px-4">
        <span className="hidden text-sm font-semibold md:inline">CineVault</span>
        <Clapperboard className="size-4 text-cv-accent md:hidden" aria-hidden />
      </div>
      <div className="flex flex-1 flex-col gap-1 px-2">
        {items.map((item) => {
          const Icon = item.icon;
          const active = view === item.id;
          return (
            <button
              key={item.id}
              type="button"
              aria-current={active ? "page" : undefined}
              aria-label={item.label}
              onClick={() => onViewChange(item.id)}
              className={cn(
                "flex h-9 items-center justify-center gap-2 rounded-md px-2 text-sm md:justify-start",
                active
                  ? "bg-cv-surface-raised text-cv-accent"
                  : "text-cv-muted hover:bg-cv-surface-raised hover:text-cv-foreground",
              )}
            >
              <Icon className="size-4 shrink-0" aria-hidden />
              <span className="hidden md:inline">{item.label}</span>
              {item.id === "duplicates" && duplicateCount > 0 ? (
                <span className="ml-auto hidden tabular-nums text-xs text-cv-muted md:inline">
                  {duplicateCount}
                </span>
              ) : null}
            </button>
          );
        })}
      </div>
      <div className="border-t border-cv-border p-2">
        <button
          type="button"
          aria-label={theme === "dark" ? "Switch to day theme" : "Switch to night theme"}
          onClick={onToggleTheme}
          className="flex h-9 w-full items-center justify-center gap-2 rounded-md text-sm text-cv-muted hover:bg-cv-surface-raised hover:text-cv-foreground md:justify-start md:px-2"
        >
          {theme === "dark" ? (
            <Sun className="size-4" aria-hidden />
          ) : (
            <Moon className="size-4" aria-hidden />
          )}
          <span className="hidden md:inline">{theme === "dark" ? "Day" : "Night"}</span>
        </button>
      </div>
    </nav>
  );
}