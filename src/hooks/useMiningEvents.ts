import { useEffect, useRef, useState, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { MiningStatus, LogEntry } from "@/lib/invoke";

export function useMiningEvents() {
  const [miningStatus, setMiningStatus] = useState<MiningStatus>({
    running: false,
    active: false,
    click_count: 0,
    detection_count: 0,
  });
  const [miningLogs, setMiningLogs] = useState<LogEntry[]>([]);

  const mounted = useRef(false);

  const clearMiningLogs = useCallback(() => setMiningLogs([]), []);

  useEffect(() => {
    if (mounted.current) return;
    mounted.current = true;

    const unsubs: UnlistenFn[] = [];

    (async () => {
      unsubs.push(
        await listen<MiningStatus>("mining-status", (e) =>
          setMiningStatus(e.payload)
        )
      );
      unsubs.push(
        await listen<LogEntry>("mining-log", (e) =>
          setMiningLogs((prev) => {
            const next = [...prev, e.payload];
            return next.length > 500 ? next.slice(-500) : next;
          })
        )
      );
    })();

    return () => {
      unsubs.forEach((u) => u());
      mounted.current = false;
    };
  }, []);

  return { miningStatus, miningLogs, clearMiningLogs };
}
