import { useState, useEffect, useCallback } from "react";
import {
  getConfig,
  startAutomation,
  stopAutomation,
  type AutomationStatus,
  type LogEntry,
  type WindowStatus,
} from "@/lib/invoke";
import StatusIndicator from "@/components/StatusIndicator";
import LogViewer from "@/components/LogViewer";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Play,
  Square,
  Monitor,
  MonitorOff,
  RefreshCw,
  Clock,
  AlertTriangle,
} from "lucide-react";

interface Props {
  status: AutomationStatus;
  logs: LogEntry[];
  windowInfo: WindowStatus;
  onClearLogs: () => void;
}

export default function Dashboard({
  status,
  logs,
  windowInfo,
  onClearLogs,
}: Props) {
  const [charName, setCharName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    getConfig().then((c) => setCharName(c.character_name));
  }, []);

  const handleStart = useCallback(async () => {
    if (!charName.trim()) {
      setError("Configure d'abord le nom du personnage dans les paramètres.");
      return;
    }
    setError("");
    setLoading(true);
    try {
      await startAutomation(charName);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [charName]);

  const handleStop = useCallback(async () => {
    try {
      await stopAutomation();
    } catch (e) {
      setError(String(e));
    }
  }, []);

  return (
    <div className="flex h-full">
      {/* ── Left: Status ── */}
      <div className="flex flex-col w-[340px] shrink-0 border-r border-border">
        {/* Status area */}
        <div className="flex-1 flex flex-col items-center justify-center px-6 gap-2">
          <StatusIndicator status={status} />

          {/* Big button */}
          <div className="w-full max-w-56 mt-2">
            {!status.running ? (
              <Button
                onClick={handleStart}
                disabled={loading}
                className="w-full h-11 gap-2 text-sm font-bold bg-gradient-to-r from-emerald-500 to-green-600 text-white hover:from-emerald-400 hover:to-green-500 shadow-lg shadow-emerald-500/20 border-0"
                size="lg"
              >
                <Play className="size-4" fill="white" />
                {loading ? "Démarrage..." : "Démarrer"}
              </Button>
            ) : (
              <Button
                onClick={handleStop}
                variant="destructive"
                className="w-full h-11 gap-2 text-sm font-bold"
                size="lg"
              >
                <Square className="size-4" fill="currentColor" />
                Arrêter
              </Button>
            )}
          </div>

          {error && (
            <div className="flex items-start gap-2 mt-2 p-2.5 rounded-lg bg-destructive/10 border border-destructive/20 max-w-56 w-full">
              <AlertTriangle className="size-3.5 text-destructive shrink-0 mt-0.5" />
              <p className="text-[11px] text-destructive leading-tight">{error}</p>
            </div>
          )}
        </div>

        {/* Stats */}
        <div className="px-4 pb-4 space-y-2">
          <Separator />
          <div className="grid gap-2 pt-2">
            {/* Window */}
            <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-secondary/50">
              {windowInfo.found ? (
                <Monitor className="size-4 text-emerald-400" />
              ) : (
                <MonitorOff className="size-4 text-muted-foreground" />
              )}
              <div className="flex-1 min-w-0">
                <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Fenêtre
                </p>
                <p className="text-xs font-medium truncate">
                  {windowInfo.found
                    ? `${windowInfo.width} × ${windowInfo.height}`
                    : "Non détectée"}
                </p>
              </div>
              {windowInfo.found && (
                <Badge variant="secondary" className="text-[9px] text-emerald-400">
                  ON
                </Badge>
              )}
            </div>

            {/* Reconnections */}
            <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-secondary/50">
              <RefreshCw className="size-4 text-muted-foreground" />
              <div className="flex-1">
                <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Reconnexions
                </p>
                <p className="text-xs font-medium tabular-nums">
                  {status.reconnect_count}
                </p>
              </div>
            </div>

            {/* Last crash */}
            {status.last_crash_time && (
              <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-secondary/50">
                <Clock className="size-4 text-amber-400" />
                <div className="flex-1 min-w-0">
                  <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                    Dernier crash
                  </p>
                  <p className="text-xs font-medium text-amber-300 truncate">
                    {status.last_crash_time}
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* ── Right: Logs ── */}
      <div className="flex-1 min-w-0 p-3">
        <LogViewer logs={logs} onClear={onClearLogs} />
      </div>
    </div>
  );
}
