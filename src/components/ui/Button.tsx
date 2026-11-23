import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "../../lib/cn";

type Variant = "primary" | "secondary" | "danger" | "ghost";
type Size = "md" | "sm";

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: Variant;
  size?: Size;
  icon?: ReactNode;
};

export function Button({
  variant = "secondary",
  size = "md",
  icon,
  className,
  children,
  type = "button",
  ...props
}: Props) {
  return (
    <button
      type={type}
      className={cn(
        "inline-flex items-center justify-center gap-2 rounded-md font-medium",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cv-accent-muted",
        "active:scale-[0.98] disabled:pointer-events-none disabled:opacity-40",
        size === "md" ? "h-9 px-3 text-sm" : "h-7 px-2 text-xs",
        variant === "primary" && "bg-cv-accent text-white hover:bg-cv-accent/90",
        variant === "secondary" &&
          "border border-cv-border bg-transparent text-cv-foreground hover:bg-cv-surface-raised",
        variant === "danger" &&
          "border border-cv-danger/40 text-cv-danger hover:bg-cv-surface-raised",
        variant === "ghost" &&
          "text-cv-muted hover:bg-cv-surface-raised hover:text-cv-foreground",
        className,
      )}
      {...props}
    >
      {icon}
      {children}
    </button>
  );
}