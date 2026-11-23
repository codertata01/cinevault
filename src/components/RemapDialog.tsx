import { useState } from "react";
import type { RemapCandidate } from "../types/api";
import { Button } from "./ui/Button";
import { Dialog } from "./ui/Dialog";

type RemapDialogProps = {
  candidate: RemapCandidate;
  onConfirm: (autoRemap: boolean) => void;
  onDismiss: () => void;
  busy?: boolean;
};

export function RemapDialog({
  candidate,
  onConfirm,
  onDismiss,
  busy = false,
}: RemapDialogProps) {
  const [alwaysRemap, setAlwaysRemap] = useState(false);

  return (
    <Dialog
      title="Remap drive letter?"
      titleId="remap-title"
      actions={
        <>
          <Button onClick={onDismiss} disabled={busy}>
            Not now
          </Button>
          <Button variant="primary" disabled={busy} onClick={() => onConfirm(alwaysRemap)}>
            Remap
          </Button>
        </>
      }
    >
      <p>
        <span className="text-cv-foreground">{candidate.label}</span> moved from{" "}
        <span className="tabular-nums text-cv-foreground">{candidate.previousDriveLetter}</span> to{" "}
        <span className="tabular-nums text-cv-foreground">{candidate.currentDriveLetter}</span>.
        Update stored paths for this volume?
      </p>
      <label className="mt-4 flex items-center gap-2">
        <input
          type="checkbox"
          checked={alwaysRemap}
          onChange={(event) => setAlwaysRemap(event.target.checked)}
          className="size-4 rounded border-cv-border"
        />
        Always remap this drive automatically
      </label>
    </Dialog>
  );
}