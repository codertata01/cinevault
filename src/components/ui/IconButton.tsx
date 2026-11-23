import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "../../lib/cn";

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
  label: string;
  icon: ReactNode;
  tone?: "default" | "danger";
};

export function IconButton({
  label,
  icon,
  tone = "default",
  className,
  type = "button",
  ...props
}: Props) {
  return (
    <button
      type={type}
      aria-label={label}
      title={label}
      className={cn(
        "inline-flex size-8 items-center justify-center rounded-md",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cv-accent-muted",
        "active:scale-[0.98] disabled:pointer-events-none disabled:opacity-40",
        tone === "danger"
          ? "text-cv-danger hover:bg-cv-surface-raised"
          : "text-cv-muted hover:bg-cv-surface-raised hover:text-cv-foreground",
        className,
      )}
      {...props}
    >
      {icon}
    </button>
  );
}