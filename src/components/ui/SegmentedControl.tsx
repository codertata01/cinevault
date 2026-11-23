import type { ReactNode } from "react";
import { cn } from "../../lib/cn";

export type Segment<T extends string> = {
  id: T;
  label: string;
  icon?: ReactNode;
  count?: number;
};

type Props<T extends string> = {
  label: string;
  value: T;
  items: Segment<T>[];
  onChange: (value: T) => void;
  iconOnly?: boolean;
};

export function SegmentedControl<T extends string>({
  label,
  value,
  items,
  onChange,
  iconOnly = false,
}: Props<T>) {
  return (
    <div
      role="group"
      aria-label={label}
      className="inline-flex h-8 items-center rounded-lg border border-cv-border bg-cv-bg p-0.5"
    >
      {items.map((item) => {
        const selected = item.id === value;
        return (
          <button
            key={item.id}
            type="button"
            aria-pressed={selected}
            aria-label={iconOnly ? item.label : undefined}
            title={iconOnly ? item.label : undefined}
            onClick={() => onChange(item.id)}
            className={cn(
              "inline-flex h-7 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium",
              selected
                ? "bg-cv-surface-raised text-cv-foreground"
                : "text-cv-muted hover:text-cv-foreground",
            )}
          >
            {item.icon}
            {iconOnly ? null : <span>{item.label}</span>}
            {item.count != null ? (
              <span className="inline-block min-w-[2.75rem] text-right tabular-nums text-cv-muted">
                {item.count.toLocaleString()}
              </span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}