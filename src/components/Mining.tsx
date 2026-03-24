import { useCallback, useEffect, useState } from "react";
import {
  startMining,
  stopMining,
  getMiningConfig,
  saveMiningConfig,
  type MiningConfig,
  type MiningStatus,
  type LogEntry,
} from "@/lib/invoke";
import LogViewer from "@/components/LogViewer";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import {
  Play,
  Square,
  Gem,
  MousePointerClick,
  Eye,
  ChevronDown,
  ChevronUp,
  AlertTriangle,
  Save,
} from "lucide-react";

interface Props {
  miningStatus: MiningStatus;
  miningLogs: LogEntry[];
  onClearLogs: () => void;
}

const DEFAULT_CONFIG: MiningConfig = {
  target_color: [255, 3, 14],
  tolerance: 30,
  margin: 0.2,
  min_delay: 0.1,
  max_delay: 0.3,
  max_distance: 600,
  toggle_key: "M",
};

function rgbToHex(r: number, g: number, b: number): string {
  return (
    "#" +
    [r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("")
  );
}

function hexToRgb(hex: string): [number, number, number] {
  const h = hex.replace("#", "");
  return [
    parseInt(h.substring(0, 2), 16) || 0,
    parseInt(h.substring(2, 4), 16) || 0,
    parseInt(h.substring(4, 6), 16) || 0,
  ];
}

export default function Mining({
  miningStatus,
  miningLogs,
  onClearLogs,
}: Props) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [config, setConfig] = useState<MiningConfig>(DEFAULT_CONFIG);
  const [showConfig, setShowConfig] = useState(false);
  const [configSaved, setConfigSaved] = useState(false);

  useEffect(() => {
    getMiningConfig()
      .then(setConfig)
      .catch(() => setConfig(DEFAULT_CONFIG));
  }, []);

  const handleStart = useCallback(async () => {
    setError("");
    setLoading(true);
    try {
      await saveMiningConfig(config);
      await startMining(config);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [config]);

  const handleStop = useCallback(async () => {
    try {
      await stopMining();
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const handleSaveConfig = useCallback(async () => {
    try {
      await saveMiningConfig(config);
      setConfigSaved(true);
      setTimeout(() => setConfigSaved(false), 2000);
    } catch (e) {
      setError(String(e));
    }
  }, [config]);

  const running = miningStatus.running;

  return (
    <div className="flex h-full">
      {/* Left panel */}
      <div className="flex flex-col w-[340px] shrink-0 border-r border-border overflow-y-auto">
        <div className="flex-1 flex flex-col items-center justify-center px-6 gap-2">
          {/* Status orb */}
          <div className="relative flex items-center justify-center size-40 shrink-0">
            {running && (
              <>
                <div className="absolute inset-0 rounded-full bg-gradient-to-br from-amber-500 to-orange-600 opacity-20 animate-pulse-glow" />
                <div
                  className="absolute inset-5 rounded-full bg-gradient-to-br from-amber-500 to-orange-600 opacity-25 animate-pulse-glow"
                  style={{ animationDelay: "0.6s" }}
                />
              </>
            )}
            <div
              className={cn(
                "relative size-[4.5rem] rounded-full bg-gradient-to-br flex items-center justify-center shadow-xl transition-all duration-500",
                running
                  ? miningStatus.active
                    ? "from-amber-500 to-orange-600"
                    : "from-amber-700 to-orange-800"
                  : "from-zinc-600 to-zinc-700"
              )}
            >
              <Gem
                className={cn(
                  "size-8",
                  running ? "text-white" : "text-white/70"
                )}
              />
            </div>
          </div>

          {/* Label */}
          <div className="text-center space-y-1 shrink-0">
            <p className="text-base font-semibold tracking-tight">
              {running
                ? miningStatus.active
                  ? "Mining actif"
                  : "Mining en pause"
                : "Mining en attente"}
            </p>
            {running && (
              <p className="text-[11px] text-muted-foreground">
                Touche {config.toggle_key} pour pause/reprise
              </p>
            )}
          </div>

          {/* Button */}
          <div className="w-full max-w-56 mt-2 shrink-0">
            {!running ? (
              <Button
                onClick={handleStart}
                disabled={loading}
                className="w-full h-11 gap-2 text-sm font-bold bg-gradient-to-r from-amber-500 to-orange-600 text-white hover:from-amber-400 hover:to-orange-500 shadow-lg shadow-amber-500/20 border-0"
                size="lg"
              >
                <Play className="size-4" fill="white" />
                {loading ? "Démarrage..." : "Démarrer Mining"}
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
            <div className="flex items-start gap-2 mt-2 p-2.5 rounded-lg bg-destructive/10 border border-destructive/20 max-w-56 w-full shrink-0">
              <AlertTriangle className="size-3.5 text-destructive shrink-0 mt-0.5" />
              <p className="text-[11px] text-destructive leading-tight">
                {error}
              </p>
            </div>
          )}
        </div>

        {/* Stats + Config */}
        <div className="px-4 pb-4 space-y-2">
          <Separator />
          <div className="grid gap-2 pt-2">
            <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-secondary/50">
              <MousePointerClick className="size-4 text-amber-400" />
              <div className="flex-1">
                <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Clics effectués
                </p>
                <p className="text-xs font-medium tabular-nums">
                  {miningStatus.click_count}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-secondary/50">
              <Eye className="size-4 text-amber-400" />
              <div className="flex-1">
                <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Détections
                </p>
                <p className="text-xs font-medium tabular-nums">
                  {miningStatus.detection_count}
                </p>
              </div>
            </div>
          </div>

          {/* Config toggle */}
          <button
            onClick={() => setShowConfig(!showConfig)}
            disabled={running}
            className="flex items-center justify-between w-full px-3 py-2 rounded-lg bg-secondary/50 hover:bg-secondary/80 transition-colors text-xs text-muted-foreground disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <span>Configuration</span>
            {showConfig ? (
              <ChevronUp className="size-3.5" />
            ) : (
              <ChevronDown className="size-3.5" />
            )}
          </button>

          {showConfig && !running && (
            <div className="space-y-3 px-1 pt-1">
              {/* Color */}
              <div className="space-y-1">
                <Label className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Couleur cible
                </Label>
                <div className="flex items-center gap-2">
                  <input
                    type="color"
                    value={rgbToHex(
                      config.target_color[0],
                      config.target_color[1],
                      config.target_color[2]
                    )}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        target_color: hexToRgb(e.target.value),
                      })
                    }
                    className="size-8 rounded border border-border cursor-pointer bg-transparent"
                  />
                  <span className="text-[11px] text-muted-foreground font-mono">
                    {rgbToHex(
                      config.target_color[0],
                      config.target_color[1],
                      config.target_color[2]
                    )}
                  </span>
                </div>
              </div>

              {/* Tolerance */}
              <div className="space-y-1">
                <Label className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Tolérance (±)
                </Label>
                <Input
                  type="number"
                  min={0}
                  max={255}
                  value={config.tolerance}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      tolerance: Number(e.target.value),
                    })
                  }
                  className="h-7 text-xs"
                />
              </div>

              {/* Margin */}
              <div className="space-y-1">
                <Label className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Marge écran (%)
                </Label>
                <Input
                  type="number"
                  min={0}
                  max={49}
                  step={1}
                  value={Math.round(config.margin * 100)}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      margin: Number(e.target.value) / 100,
                    })
                  }
                  className="h-7 text-xs"
                />
              </div>

              {/* Delays */}
              <div className="grid grid-cols-2 gap-2">
                <div className="space-y-1">
                  <Label className="text-[10px] uppercase tracking-wider text-muted-foreground">
                    Délai min (s)
                  </Label>
                  <Input
                    type="number"
                    min={0.01}
                    max={5}
                    step={0.01}
                    value={config.min_delay}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        min_delay: Number(e.target.value),
                      })
                    }
                    className="h-7 text-xs"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] uppercase tracking-wider text-muted-foreground">
                    Délai max (s)
                  </Label>
                  <Input
                    type="number"
                    min={0.01}
                    max={5}
                    step={0.01}
                    value={config.max_delay}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        max_delay: Number(e.target.value),
                      })
                    }
                    className="h-7 text-xs"
                  />
                </div>
              </div>

              {/* Max distance */}
              <div className="space-y-1">
                <Label className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Distance max (px)
                </Label>
                <Input
                  type="number"
                  min={50}
                  max={5000}
                  value={config.max_distance}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      max_distance: Number(e.target.value),
                    })
                  }
                  className="h-7 text-xs"
                />
              </div>

              {/* Toggle key */}
              <div className="space-y-1">
                <Label className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Touche toggle
                </Label>
                <Input
                  value={config.toggle_key}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      toggle_key: e.target.value.toUpperCase(),
                    })
                  }
                  maxLength={5}
                  className="h-7 text-xs uppercase"
                  placeholder="M"
                />
              </div>

              {/* Save */}
              <Button
                onClick={handleSaveConfig}
                variant="secondary"
                size="sm"
                className="w-full h-8 gap-1.5 text-xs"
              >
                <Save className="size-3" />
                {configSaved ? "Sauvegardé !" : "Sauvegarder config"}
              </Button>
            </div>
          )}
        </div>
      </div>

      {/* Right panel: logs */}
      <div className="flex-1 min-w-0 p-3">
        <LogViewer logs={miningLogs} onClear={onClearLogs} />
      </div>
    </div>
  );
}
