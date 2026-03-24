import { useCallback, useState } from "react";
import {
  startMuscu,
  stopMuscu,
  type MuscuStatus,
  type LogEntry,
} from "@/lib/invoke";
import LogViewer from "@/components/LogViewer";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import {
  Play,
  Square,
  Dumbbell,
  RotateCw,
  AlertTriangle,
} from "lucide-react";

interface Props {
  muscuStatus: MuscuStatus;
  muscuLogs: LogEntry[];
  onClearLogs: () => void;
}

export default function Muscu({ muscuStatus, muscuLogs, onClearLogs }: Props) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleStart = useCallback(async () => {
    setError("");
    setLoading(true);
    try {
      await startMuscu();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const handleStop = useCallback(async () => {
    try {
      await stopMuscu();
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const active = muscuStatus.running;

  return (
    <div className="flex h-full">
      {/* Left panel */}
      <div className="flex flex-col w-[340px] shrink-0 border-r border-border">
        <div className="flex-1 flex flex-col items-center justify-center px-6 gap-2">
          {/* Status orb */}
          <div className="relative flex items-center justify-center size-40">
            {active && (
              <>
                <div className="absolute inset-0 rounded-full bg-gradient-to-br from-violet-500 to-purple-600 opacity-20 animate-pulse-glow" />
                <div
                  className="absolute inset-5 rounded-full bg-gradient-to-br from-violet-500 to-purple-600 opacity-25 animate-pulse-glow"
                  style={{ animationDelay: "0.6s" }}
                />
              </>
            )}
            <div
              className={cn(
                "relative size-[4.5rem] rounded-full bg-gradient-to-br flex items-center justify-center shadow-xl transition-all duration-500",
                active
                  ? "from-violet-500 to-purple-600"
                  : "from-zinc-600 to-zinc-700"
              )}
            >
              <Dumbbell
                className={cn(
                  "size-8",
                  active ? "text-white" : "text-white/70"
                )}
              />
            </div>
          </div>

          {/* Label */}
          <div className="text-center space-y-2">
            <p className="text-base font-semibold tracking-tight">
              {active ? "Muscu AFK actif" : "Muscu AFK en attente"}
            </p>
          </div>

          {/* Button */}
          <div className="w-full max-w-56 mt-2">
            {!active ? (
              <Button
                onClick={handleStart}
                disabled={loading}
                className="w-full h-11 gap-2 text-sm font-bold bg-gradient-to-r from-violet-500 to-purple-600 text-white hover:from-violet-400 hover:to-purple-500 shadow-lg shadow-violet-500/20 border-0"
                size="lg"
              >
                <Play className="size-4" fill="white" />
                {loading ? "Démarrage..." : "Démarrer Muscu"}
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
              <p className="text-[11px] text-destructive leading-tight">
                {error}
              </p>
            </div>
          )}
        </div>

        {/* Stats */}
        <div className="px-4 pb-4 space-y-2">
          <Separator />
          <div className="grid gap-2 pt-2">
            <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-secondary/50">
              <RotateCw className="size-4 text-violet-400" />
              <div className="flex-1">
                <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Cycles complétés
                </p>
                <p className="text-xs font-medium tabular-nums">
                  {muscuStatus.cycle_count}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Right panel: logs */}
      <div className="flex-1 min-w-0 p-3">
        <LogViewer logs={muscuLogs} onClear={onClearLogs} />
      </div>
    </div>
  );
}
