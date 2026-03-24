import { useEffect, useRef, useState, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AutomationStatus, LogEntry, WindowStatus } from "@/lib/invoke";

export function useTauriEvents() {
  const [status, setStatus] = useState<AutomationStatus>({
    state: "idle",
    running: false,
    session_treated: false,
    reconnect_count: 0,
    afk_start_time: null,
    last_crash_time: null,
  });
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [windowInfo, setWindowInfo] = useState<WindowStatus>({
    found: false,
    title: "",
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    hwnd: 0,
  });

  // Guard against double-mount in dev
  const mounted = useRef(false);

  const clearLogs = useCallback(() => setLogs([]), []);

  useEffect(() => {
    if (mounted.current) return;
    mounted.current = true;

    const unsubs: UnlistenFn[] = [];

    (async () => {
      unsubs.push(
        await listen<AutomationStatus>("automation:status", (e) =>
          setStatus(e.payload)
        )
      );
      unsubs.push(
        await listen<LogEntry>("automation:log", (e) =>
          setLogs((prev) => {
            const next = [...prev, e.payload];
            return next.length > 500 ? next.slice(-500) : next;
          })
        )
      );
      unsubs.push(
        await listen<WindowStatus>("automation:window", (e) =>
          setWindowInfo(e.payload)
        )
      );
    })();

    return () => {
      unsubs.forEach((u) => u());
      mounted.current = false;
    };
  }, []);

  return { status, setStatus, logs, clearLogs, windowInfo };
}
