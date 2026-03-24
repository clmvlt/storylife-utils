import { useEffect, useRef, useState, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { MuscuStatus, LogEntry } from "@/lib/invoke";

export function useMuscuEvents() {
  const [muscuStatus, setMuscuStatus] = useState<MuscuStatus>({
    running: false,
    cycle_count: 0,
  });
  const [muscuLogs, setMuscuLogs] = useState<LogEntry[]>([]);

  const mounted = useRef(false);

  const clearMuscuLogs = useCallback(() => setMuscuLogs([]), []);

  useEffect(() => {
    if (mounted.current) return;
    mounted.current = true;

    const unsubs: UnlistenFn[] = [];

    (async () => {
      unsubs.push(
        await listen<MuscuStatus>("muscu-status", (e) =>
          setMuscuStatus(e.payload)
        )
      );
      unsubs.push(
        await listen<LogEntry>("muscu-log", (e) =>
          setMuscuLogs((prev) => {
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

  return { muscuStatus, muscuLogs, clearMuscuLogs };
}
