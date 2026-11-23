import { useEffect, useRef } from "react";

type Options = {
  enabled: boolean;
  loading: boolean;
  /** Inclusive index of the last rendered item (or row for grid). */
  lastVisibleIndex: number;
  totalCount: number;
  /** How many items from the end should trigger a load. */
  threshold?: number;
  onLoadMore: () => void;
};

/** Fires `onLoadMore` once when the user scrolls within `threshold` of the end. */
export function useInfiniteLoadNearEnd({
  enabled,
  loading,
  lastVisibleIndex,
  totalCount,
  threshold = 12,
  onLoadMore,
}: Options) {
  const pendingRef = useRef(false);

  useEffect(() => {
    if (!enabled || loading || totalCount === 0) {
      return;
    }
    if (lastVisibleIndex < totalCount - threshold) {
      pendingRef.current = false;
      return;
    }
    if (pendingRef.current) {
      return;
    }
    pendingRef.current = true;
    onLoadMore();
  }, [enabled, lastVisibleIndex, loading, onLoadMore, threshold, totalCount]);

  useEffect(() => {
    if (!loading) {
      pendingRef.current = false;
    }
  }, [loading]);
}