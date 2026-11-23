import { ChevronDown, ChevronRight, FolderMinus, FolderPlus, HardDrive } from "lucide-react";
import { useState } from "react";
import type { DataSourceRecord } from "../../types/api";
import { Button } from "../ui/Button";
import { IconButton } from "../ui/IconButton";
import { StatusPill } from "../ui/StatusPill";

type Props = {
  sources: DataSourceRecord[];
  busy: boolean;
  defaultOpen: boolean;
  onAdd: () => void;
  onSync: () => void;
  onScan: (id: number) => void;
  onRemove: (source: DataSourceRecord) => void;
};

export function DataSourcesPanel({
  sources,
  busy,
  defaultOpen,
  onAdd,
  onSync,
  onScan,
  onRemove,
}: Props) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <section className="mb-3 rounded-xl border border-cv-border bg-cv-surface">
      <div className="flex items-center gap-2 px-3 py-2">
        <button
          type="button"
          aria-expanded={open}
          onClick={() => setOpen((value) => !value)}
          className="flex min-w-0 flex-1 items-center gap-2 text-left text-sm"
        >
          {open ? (
            <ChevronDown className="size-4 text-cv-muted" aria-hidden />
          ) : (
            <ChevronRight className="size-4 text-cv-muted" aria-hidden />
          )}
          <span className="font-medium">Folders</span>
          <span className="tabular-nums text-xs text-cv-muted">{sources.length}</span>
        </button>
        <Button
          size="sm"
          icon={<HardDrive className="size-3.5" />}
          disabled={busy}
          onClick={onSync}
        >
          Sync
        </Button>
        <Button
          size="sm"
          icon={<FolderPlus className="size-3.5" />}
          disabled={busy}
          onClick={onAdd}
        >
          Add folder
        </Button>
      </div>
      {open ? (
        sources.length === 0 ? (
          <p className="text-pretty border-t border-cv-border px-3 py-3 text-sm text-cv-muted">
            Add a movie folder, then scan the library.
          </p>
        ) : (
          <ul className="border-t border-cv-border">
            {sources.map((source) => (
              <li
                key={source.id}
                className="flex items-center gap-3 border-b border-cv-border px-3 py-2 last:border-b-0"
              >
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm">
                    <span className="font-mono tabular-nums text-cv-muted">
                      {source.driveLetter ?? "—"}
                    </span>{" "}
                    {source.rootPath}
                  </p>
                </div>
                <StatusPill status={source.volumeOnline ? "online" : "offline"} />
                <Button
                  size="sm"
                  disabled={busy || !source.driveLetter}
                  onClick={() => onScan(source.id)}
                >
                  Scan
                </Button>
                <IconButton
                  label={`Remove ${source.rootPath}`}
                  tone="danger"
                  disabled={busy}
                  icon={<FolderMinus className="size-4" />}
                  onClick={() => onRemove(source)}
                />
              </li>
            ))}
          </ul>
        )
      ) : null}
    </section>
  );
}