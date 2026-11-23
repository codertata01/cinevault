import type { ReactNode } from "react";
import { cn } from "../../lib/cn";

export function Badge({
  children,
  mono = false,
  className,
}: {
  children: ReactNode;
  mono?: boolean;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex w-fit max-w-full shrink-0 items-center rounded-md border border-cv-border bg-cv-surface-raised px-1.5 text-xs text-cv-foreground",
        mono && "font-mono tabular-nums",
        className,
      )}
    >
      {children}
    </span>
  );
}