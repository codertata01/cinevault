import type { ReactNode } from "react";

type Props = {
  title: string;
  titleId: string;
  children: ReactNode;
  actions: ReactNode;
};

export function Dialog({ title, titleId, children, actions }: Props) {
  return (
    <div
      className="fixed inset-0 z-30 flex items-center justify-center bg-black/55 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
    >
      <div className="w-full max-w-md rounded-xl border border-cv-border bg-cv-surface p-5">
        <h2 id={titleId} className="text-balance text-base font-semibold">
          {title}
        </h2>
        <div className="text-pretty mt-2 text-sm text-cv-muted">{children}</div>
        <div className="mt-5 flex justify-end gap-2">{actions}</div>
      </div>
    </div>
  );
}