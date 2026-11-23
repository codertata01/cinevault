import { cn } from "../../lib/cn";

export function StatusPill({ status }: { status: string }) {
  const offline = status === "offline";
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 text-xs capitalize",
        offline ? "text-cv-danger" : "text-cv-success",
      )}
    >
      <span
        className={cn(
          "size-2 rounded-full",
          offline ? "bg-cv-danger" : "bg-cv-success",
        )}
        aria-hidden
      />
      {status}
    </span>
  );
}