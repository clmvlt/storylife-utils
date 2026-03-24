import { useState, useCallback, useEffect } from "react";
import {
  getConfig,
  manualKillFivem,
  manualLaunchFivem,
  manualFindWindow,
  manualOcrDetect,
  manualClickCharacter,
  manualClickSuivant,
  manualFocusWindow,
  manualSendAfkKeys,
  type WindowStatus,
  type OcrResult,
  type LogEntry,
} from "@/lib/invoke";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import LogViewer from "@/components/LogViewer";
import {
  Skull,
  Rocket,
  Search,
  ScanEye,
  MousePointerClick,
  ArrowRight,
  Focus,
  Keyboard,
  Loader2,
  CheckCircle2,
  XCircle,
  ChevronRight,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface Props {
  logs: LogEntry[];
  onClearLogs: () => void;
}

interface StepState {
  loading: boolean;
  result: string | null;
  error: string | null;
}

const INITIAL: StepState = { loading: false, result: null, error: null };

export default function DebugSteps({ logs, onClearLogs }: Props) {
  const [charName, setCharName] = useState("");
  const [windowInfo, setWindowInfo] = useState<WindowStatus | null>(null);
  const [ocrResult, setOcrResult] = useState<OcrResult | null>(null);
  const [steps, setSteps] = useState<Record<string, StepState>>({});

  useEffect(() => {
    getConfig().then((c) => setCharName(c.character_name));
  }, []);

  const runStep = useCallback(
    async (key: string, fn: () => Promise<string | unknown>) => {
      setSteps((s) => ({ ...s, [key]: { loading: true, result: null, error: null } }));
      try {
        const res = await fn();
        const msg = typeof res === "string" ? res : "OK";
        setSteps((s) => ({ ...s, [key]: { loading: false, result: msg, error: null } }));
        return res;
      } catch (e) {
        const msg = String(e);
        setSteps((s) => ({ ...s, [key]: { loading: false, result: null, error: msg } }));
        return null;
      }
    },
    []
  );

  const st = (key: string) => steps[key] ?? INITIAL;

  const STEP_LIST: {
    key: string;
    num: number;
    icon: React.ReactNode;
    title: string;
    desc: string;
    action: () => Promise<unknown>;
    disabled?: boolean;
  }[] = [
    {
      key: "kill",
      num: 1,
      icon: <Skull className="size-4" />,
      title: "Kill FiveM",
      desc: "Tue tous les processus FiveM existants",
      action: () => runStep("kill", manualKillFivem),
    },
    {
      key: "launch",
      num: 2,
      icon: <Rocket className="size-4" />,
      title: "Lancer FiveM",
      desc: "Ouvre FiveM et se connecte à StoryLife",
      action: () => runStep("launch", manualLaunchFivem),
    },
    {
      key: "find",
      num: 3,
      icon: <Search className="size-4" />,
      title: "Trouver la fenêtre",
      desc: "Recherche la fenêtre FiveM (titre contient storylife + fivem)",
      action: async () => {
        const res = await runStep("find", manualFindWindow);
        if (res && typeof res === "object") setWindowInfo(res as WindowStatus);
      },
    },
    {
      key: "ocr",
      num: 4,
      icon: <ScanEye className="size-4" />,
      title: "Détecter le personnage (OCR)",
      desc: `Recherche "${charName || "..."}" via OCR dans la fenêtre`,
      action: async () => {
        const res = await runStep("ocr", () => manualOcrDetect(charName));
        if (res && typeof res === "object") setOcrResult(res as OcrResult);
      },
      disabled: !charName,
    },
    {
      key: "click_char",
      num: 5,
      icon: <MousePointerClick className="size-4" />,
      title: "Cliquer le personnage",
      desc: "Clique sur la position OCR détectée",
      action: () =>
        runStep("click_char", () =>
          manualClickCharacter(
            windowInfo?.x ?? 0,
            windowInfo?.y ?? 0,
            ocrResult?.x ?? 0,
            ocrResult?.y ?? 0,
            ocrResult?.width ?? 0,
            ocrResult?.height ?? 0
          )
        ),
      disabled: !ocrResult?.found || !windowInfo,
    },
    {
      key: "click_suivant",
      num: 6,
      icon: <ArrowRight className="size-4" />,
      title: 'Cliquer "Suivant"',
      desc: "Clique le bouton Suivant (position fixe)",
      action: () =>
        runStep("click_suivant", () =>
          manualClickSuivant(
            windowInfo?.x ?? 0,
            windowInfo?.y ?? 0,
            windowInfo?.height ?? 0
          )
        ),
      disabled: !windowInfo,
    },
    {
      key: "focus",
      num: 7,
      icon: <Focus className="size-4" />,
      title: "Focus fenêtre",
      desc: "Ramène la fenêtre FiveM au premier plan",
      action: () => runStep("focus", manualFocusWindow),
    },
    {
      key: "afk",
      num: 8,
      icon: <Keyboard className="size-4" />,
      title: "Envoyer séquence AFK",
      desc: "F5 → 4×Down → Enter → 3×Down → Enter",
      action: () => runStep("afk", manualSendAfkKeys),
    },
  ];

  return (
    <div className="flex h-full">
      {/* Left — Steps */}
      <div className="flex flex-col w-[420px] shrink-0 border-r border-border">
        <div className="flex-1 overflow-y-auto p-3 space-y-2">
          {STEP_LIST.map((step) => {
            const s = st(step.key);
            return (
              <Card
                key={step.key}
                size="sm"
                className={cn(
                  "transition-colors",
                  s.result && "ring-emerald-500/30",
                  s.error && "ring-destructive/30"
                )}
              >
                <CardContent className="flex items-center gap-3 py-2.5">
                  {/* Number */}
                  <div className="flex items-center justify-center size-7 rounded-md bg-secondary text-xs font-bold text-muted-foreground shrink-0">
                    {step.num}
                  </div>

                  {/* Info */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                      {step.icon}
                      <span className="text-xs font-semibold">{step.title}</span>
                    </div>
                    <p className="text-[10px] text-muted-foreground mt-0.5 truncate">
                      {step.desc}
                    </p>
                    {s.result && (
                      <p className="text-[10px] text-emerald-400 mt-1 flex items-center gap-1">
                        <CheckCircle2 className="size-3" />
                        {s.result}
                      </p>
                    )}
                    {s.error && (
                      <p className="text-[10px] text-destructive mt-1 flex items-center gap-1">
                        <XCircle className="size-3" />
                        {s.error}
                      </p>
                    )}
                  </div>

                  {/* Action */}
                  <Button
                    size="sm"
                    variant={s.result ? "secondary" : "default"}
                    onClick={step.action}
                    disabled={s.loading || step.disabled}
                    className="shrink-0"
                  >
                    {s.loading ? (
                      <Loader2 className="size-3 animate-spin" />
                    ) : (
                      <ChevronRight className="size-3" />
                    )}
                    {s.loading ? "..." : "Exécuter"}
                  </Button>
                </CardContent>
              </Card>
            );
          })}

          {/* Context info */}
          {(windowInfo || ocrResult?.found) && (
            <>
              <Separator className="my-3" />
              <div className="space-y-1 px-1">
                <p className="text-[10px] uppercase tracking-wider text-muted-foreground font-semibold">
                  Contexte
                </p>
                {windowInfo && (
                  <p className="text-[11px] text-muted-foreground">
                    Fenêtre: {windowInfo.width}x{windowInfo.height} à ({windowInfo.x},{windowInfo.y})
                  </p>
                )}
                {ocrResult?.found && (
                  <p className="text-[11px] text-muted-foreground">
                    OCR: "{ocrResult.text}" à ({Math.round(ocrResult.x)},{Math.round(ocrResult.y)})
                  </p>
                )}
              </div>
            </>
          )}
        </div>
      </div>

      {/* Right — Logs */}
      <div className="flex-1 min-w-0 p-3">
        <LogViewer logs={logs} onClear={onClearLogs} />
      </div>
    </div>
  );
}
