import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { ScanSearch } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { DuplicatesView } from "./components/duplicates/DuplicatesView";
import { LibraryView } from "./components/library/LibraryView";
import { PageHeader } from "./components/layout/PageHeader";
import { SideNav } from "./components/layout/SideNav";
import { RemapDialog } from "./components/RemapDialog";
import { Button } from "./components/ui/Button";
import { Dialog } from "./components/ui/Dialog";
import { applyTheme, getStoredTheme, oppositeTheme, type Theme } from "./lib/theme";
import type {
  DataSourceRecord,
  LibraryCounts,
  MovieListItem,
  RemapCandidate,
  ScanProgress,
  VolumeRecord,
  VolumeSyncResult,
} from "./types/api";

type View = "library" | "duplicates";

function App() {
  const [view, setView] = useState<View>("library");
  const [schemaVersion, setSchemaVersion] = useState<number | null>(null);
  const [volumes, setVolumes] = useState<VolumeRecord[]>([]);
  const [libraryRefreshToken, setLibraryRefreshToken] = useState(0);
  const [counts, setCounts] = useState<LibraryCounts>({
    all: 0,
    duplicates: 0,
    fourK: 0,
    hd1080: 0,
    offline: 0,
  });
  const [dataSources, setDataSources] = useState<DataSourceRecord[]>([]);
  const [remapQueue, setRemapQueue] = useState<RemapCandidate[]>([]);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [theme, setTheme] = useState<Theme>(() => getStoredTheme());
  const [pendingRemove, setPendingRemove] = useState<DataSourceRecord | null>(null);
  const [pendingDelete, setPendingDelete] = useState<MovieListItem | null>(null);
  const [scanProgress, setScanProgress] = useState<ScanProgress | null>(null);

  const currentRemap = remapQueue[0] ?? null;

  const refreshLibrary = useCallback(async () => {
    const [nextCounts, nextSources] = await Promise.all([
      invoke<LibraryCounts>("library_counts"),
      invoke<DataSourceRecord[]>("list_data_sources"),
    ]);
    setCounts(nextCounts);
    setDataSources(nextSources);
    setLibraryRefreshToken((token) => token + 1);
  }, []);

  const applySync = useCallback(
    async (result: VolumeSyncResult) => {
      setVolumes(result.volumes);
      if (result.remapCandidates.length > 0) {
        setRemapQueue((current) => [...current, ...result.remapCandidates]);
      }
      await refreshLibrary();
    },
    [refreshLibrary],
  );

  useEffect(() => {
    async function bootstrap() {
      try {
        const version = await invoke<number>("schema_version");
        setSchemaVersion(version);
        const sync = await invoke<VolumeSyncResult>("sync_connected_volumes");
        await applySync(sync);
      } catch (err) {
        setStatusMessage(err instanceof Error ? err.message : String(err));
      }
    }

    void bootstrap();
  }, [applySync]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<ScanProgress>("scan-progress", (event) => {
      setScanProgress(event.payload);
    }).then((stop) => {
      unlisten = stop;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  async function handleRemapConfirm(autoRemap: boolean) {
    if (!currentRemap) {
      return;
    }

    setBusy(true);
    try {
      const sync = await invoke<VolumeSyncResult>("apply_volume_remap", {
        volumeId: currentRemap.volumeId,
        newDriveLetter: currentRemap.currentDriveLetter,
        autoRemap,
      });
      setRemapQueue((queue) => queue.slice(1));
      await applySync(sync);
      setStatusMessage(
        `Remapped ${currentRemap.label} to ${currentRemap.currentDriveLetter}`,
      );
    } catch (err) {
      setStatusMessage(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleAddDataSource() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose a movie library folder",
    });
    if (!selected || Array.isArray(selected)) {
      return;
    }

    setBusy(true);
    try {
      const added = await invoke<{
        id: number;
        created: boolean;
        nestedWarning: string | null;
      }>("add_data_source_from_path", {
        absolutePath: selected,
        label: null,
      });
      await refreshLibrary();
      setStatusMessage(
        added.nestedWarning ??
          (added.created
            ? "Data source added — use Scan library or Scan on that row."
            : "That folder is already in Data sources — use Scan to refresh."),
      );
    } catch (err) {
      setStatusMessage(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleRemoveDataSource(source: DataSourceRecord) {
    setPendingRemove(null);
    setBusy(true);
    try {
      const result = await invoke<{
        moviesRemoved: number;
        rootPath: string;
      }>("remove_data_source", { dataSourceId: source.id });
      await refreshLibrary();
      setStatusMessage(
        `Removed ${result.rootPath} — ${result.moviesRemoved} movie(s) cleared from the library.`,
      );
    } catch (err) {
      setStatusMessage(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleDeleteCopy(movie: MovieListItem) {
    setPendingDelete(null);
    setBusy(true);
    try {
      const result = await invoke<{ title: string; path: string }>("delete_duplicate_copy", {
        movieId: movie.id,
      });
      await refreshLibrary();
      setStatusMessage(`Deleted ${result.title} from disk at ${result.path}`);
    } catch (err) {
      setStatusMessage(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleScan(sourceId: number) {
    setBusy(true);
    setScanProgress(null);
    try {
      const summary = await invoke<{
        moviesFound: number;
        moviesInserted: number;
        moviesUpdated: number;
        skipped: number;
      }>("scan_data_source", { dataSourceId: sourceId, maxNfoFiles: null });
      await refreshLibrary();
      setStatusMessage(
        `Scan complete — ${summary.moviesFound} found, ${summary.moviesInserted} new, ${summary.moviesUpdated} updated${summary.skipped > 0 ? `, ${summary.skipped} skipped` : ""}`,
      );
    } catch (err) {
      setStatusMessage(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
      setScanProgress(null);
    }
  }

  async function handleScanAll() {
    if (dataSources.length === 0) {
      setStatusMessage("Add a library folder before scanning.");
      return;
    }

    setBusy(true);
    setScanProgress(null);
    try {
      const result = await invoke<{
        moviesInserted: number;
        moviesUpdated: number;
        moviesFound: number;
        failedSources: number;
        sources: Array<{
          rootPath: string;
          label: string | null;
          error: string | null;
          summary: { moviesFound: number; moviesInserted: number; moviesUpdated: number };
        }>;
      }>("scan_all_data_sources", { maxNfoFiles: null });

      await refreshLibrary();

      const parts = result.sources.map((source) => {
        const name = source.label ?? source.rootPath;
        if (source.error) {
          return `${name}: failed`;
        }
        return `${name}: ${source.summary.moviesFound} found (${source.summary.moviesInserted} new)`;
      });

      setStatusMessage(
        `Scan complete — ${result.moviesInserted} new, ${result.moviesUpdated} updated · ${parts.join(" · ")}`,
      );
    } catch (err) {
      setStatusMessage(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
      setScanProgress(null);
    }
  }

  async function handleResyncVolumes() {
    setBusy(true);
    try {
      const sync = await invoke<VolumeSyncResult>("sync_connected_volumes");
      await applySync(sync);
      setStatusMessage(`Synced ${sync.volumes.length} connected volumes.`);
    } catch (err) {
      setStatusMessage(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  const onlineDrives = volumes.filter((volume) => volume.isOnline).length;
  const statusTone =
    statusMessage &&
    (statusMessage.toLowerCase().includes("fail") ||
      statusMessage.includes("No known volume") ||
      statusMessage.includes("must start"))
      ? "danger"
      : statusMessage?.toLowerCase().includes("inside")
        ? "warning"
        : "default";

  function toggleTheme() {
    const next = oppositeTheme(theme);
    setTheme(next);
    applyTheme(next);
  }

  return (
    <div className="flex h-dvh bg-cv-bg text-cv-foreground">
      <SideNav
        view={view}
        duplicateCount={counts.duplicates}
        theme={theme}
        onViewChange={setView}
        onToggleTheme={toggleTheme}
      />
      <div className="flex min-w-0 flex-1 flex-col">
      <PageHeader
        title={view === "library" ? "Library" : "Duplicates"}
        subtitle={
          <>
            <span className="tabular-nums">{counts.all}</span> titles ·{" "}
            <span className="tabular-nums">{onlineDrives}</span> drives online
          </>
        }
        status={
          scanProgress
            ? scanProgress.total === 0
              ? `Counting folders in ${scanProgress.sourceLabel || "library"}`
              : `${scanProgress.sourceLabel || "Library"} · ${scanProgress.current} / ${scanProgress.total} · ${scanProgress.title}`
            : statusMessage
        }
        statusTone={scanProgress ? "default" : statusTone}
        progress={
          scanProgress && scanProgress.total > 0
            ? scanProgress.current / scanProgress.total
            : null
        }
        actions={
          view === "library" ? (
            <Button
              variant="primary"
              icon={<ScanSearch className="size-4" />}
              disabled={busy || dataSources.length === 0}
              onClick={() => void handleScanAll()}
            >
              Scan library
            </Button>
          ) : null
        }
      />

      <main className="flex min-h-0 flex-1 flex-col px-5 py-4">
        {view === "library" ? (
          <LibraryView
            counts={counts}
            dataSources={dataSources}
            busy={busy}
            refreshToken={libraryRefreshToken}
            onScan={(id) => void handleScan(id)}
            onRemove={setPendingRemove}
            onAdd={() => void handleAddDataSource()}
            onSync={() => void handleResyncVolumes()}
            onStatus={setStatusMessage}
          />
        ) : (
          <DuplicatesView
            refreshToken={libraryRefreshToken}
            busy={busy}
            onDeleteCopy={setPendingDelete}
            onStatus={setStatusMessage}
          />
        )}
      </main>

      <footer className="flex items-center justify-end border-t border-cv-border px-5 py-2 text-xs text-cv-muted">
        <span className="tabular-nums">
          {schemaVersion === null ? "Database: initializing…" : `Schema v${schemaVersion}`}
        </span>
      </footer>
      </div>

      {currentRemap ? (
        <RemapDialog
          candidate={currentRemap}
          busy={busy}
          onConfirm={(autoRemap) => void handleRemapConfirm(autoRemap)}
          onDismiss={() => setRemapQueue((queue) => queue.slice(1))}
        />
      ) : null}

      {pendingDelete ? (
        <Dialog
          title="Delete this copy from disk?"
          titleId="delete-copy-title"
          actions={
            <>
              <Button onClick={() => setPendingDelete(null)} disabled={busy}>
                Cancel
              </Button>
              <Button
                variant="danger"
                disabled={busy}
                onClick={() => void handleDeleteCopy(pendingDelete)}
              >
                Delete folder
              </Button>
            </>
          }
        >
          {pendingDelete.title} at{" "}
          {pendingDelete.driveLetter}
          {pendingDelete.relativePath} will be removed from the drive. Other copies in the group stay.
        </Dialog>
      ) : null}

      {pendingRemove ? (
        <Dialog
          title="Remove folder from library?"
          titleId="remove-folder-title"
          actions={
            <>
              <Button onClick={() => setPendingRemove(null)} disabled={busy}>
                Cancel
              </Button>
              <Button
                variant="danger"
                disabled={busy}
                onClick={() => void handleRemoveDataSource(pendingRemove)}
              >
                Remove
              </Button>
            </>
          }
        >
          {pendingRemove.driveLetter}
          {pendingRemove.rootPath} will leave the catalog. Files on disk stay where they are.
        </Dialog>
      ) : null}
    </div>
  );
}

export default App;