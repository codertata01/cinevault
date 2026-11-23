import type { ReactNode } from "react";

type Props = {
  title: string;
  subtitle: ReactNode;
  actions?: ReactNode;
  status?: string | null;
  statusTone?: "default" | "danger" | "warning";
  progress?: number | null;
};

export function PageHeader({
  title,
  subtitle,
  actions,
  status,
  statusTone = "default",
  progress = null,
}: Props) {
  return (
    <header className="border-b border-cv-border px-5 py-3">
      <div className="flex items-center justify-between gap-4">
        <div className="min-w-0">
          <h1 className="text-balance text-base font-semibold">{title}</h1>
          <p className="truncate text-sm text-cv-muted">{subtitle}</p>
        </div>
        <div className="flex shrink-0 items-center gap-2">{actions}</div>
      </div>
      {progress != null ? (
        <div className="mt-2 h-1 overflow-hidden rounded-full bg-cv-surface-raised">
          <div
            className="h-full origin-left bg-cv-accent"
            style={{ transform: `scaleX(${Math.min(1, Math.max(0, progress))})` }}
          />
        </div>
      ) : null}
      {status ? (
        <p
          role="status"
          className={
            statusTone === "danger"
              ? "text-pretty mt-2 text-xs text-cv-danger"
              : statusTone === "warning"
                ? "text-pretty mt-2 text-xs text-cv-warning"
                : "text-pretty mt-2 text-xs text-cv-muted"
          }
        >
          {status}
        </p>
      ) : null}
    </header>
  );
}