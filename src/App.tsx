import { useState } from "react";
import Dashboard from "@/components/Dashboard";
import Settings from "@/components/Settings";
import DebugSteps from "@/components/DebugSteps";
import Muscu from "@/components/Muscu";
import Mining from "@/components/Mining";
import { useTauriEvents } from "@/hooks/useTauriEvents";
import { useMuscuEvents } from "@/hooks/useMuscuEvents";
import { useMiningEvents } from "@/hooks/useMiningEvents";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import {
  Settings as SettingsIcon,
  Youtube,
  Github,
  Gamepad2,
  Bug,
  Dumbbell,
  Gem,
} from "lucide-react";

type Page = "dashboard" | "muscu" | "mining" | "settings" | "debug";

const TABS = [
  {
    id: "dashboard" as Page,
    label: "Dashboard",
    icon: Gamepad2,
    activeColors: "border-emerald-500 text-emerald-400",
    dotColor: "bg-emerald-500",
  },
  {
    id: "muscu" as Page,
    label: "Muscu",
    icon: Dumbbell,
    activeColors: "border-violet-500 text-violet-400",
    dotColor: "bg-violet-500",
  },
  {
    id: "mining" as Page,
    label: "Mining",
    icon: Gem,
    activeColors: "border-amber-500 text-amber-400",
    dotColor: "bg-amber-500",
  },
  {
    id: "debug" as Page,
    label: "Debug",
    icon: Bug,
    activeColors: "border-sky-500 text-sky-400",
    dotColor: "bg-sky-500",
  },
  {
    id: "settings" as Page,
    label: "Paramètres",
    icon: SettingsIcon,
    activeColors: "border-zinc-400 text-zinc-300",
    dotColor: "bg-zinc-400",
  },
];

export default function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const { status, logs, clearLogs, windowInfo } = useTauriEvents();
  const { muscuStatus, muscuLogs, clearMuscuLogs } = useMuscuEvents();
  const { miningStatus, miningLogs, clearMiningLogs } = useMiningEvents();

  const isRunning = (id: Page) => {
    if (id === "dashboard") return status.running;
    if (id === "muscu") return muscuStatus.running;
    if (id === "mining") return miningStatus.running;
    return false;
  };

  return (
    <div className="flex flex-col h-screen bg-background">
      {/* ── Header ── */}
      <header className="flex items-center justify-between px-4 h-11 border-b border-border bg-card shrink-0">
        <div className="flex items-center gap-2.5">
          <Gamepad2 className="size-4 text-primary" />
          <span className="text-sm font-bold tracking-tight">
            StoryLifeUtils
          </span>
          <Badge variant="secondary" className="text-[9px] font-semibold">
            by DIMZOU
          </Badge>
        </div>

        <div className="flex items-center gap-0.5">
          <Button
            variant="ghost"
            size="icon-sm"
            render={
              <a
                href="https://www.youtube.com/@dimzou"
                target="_blank"
                rel="noopener noreferrer"
                title="YouTube"
              />
            }
          >
            <Youtube className="size-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            render={
              <a
                href="https://github.com/clmvlt/"
                target="_blank"
                rel="noopener noreferrer"
                title="GitHub"
              />
            }
          >
            <Github className="size-3.5" />
          </Button>
        </div>
      </header>

      {/* ── Tab Bar ── */}
      <nav className="flex items-end px-2 border-b border-border bg-card/50 shrink-0">
        {TABS.map((tab) => {
          const active = page === tab.id;
          const running = isRunning(tab.id);
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              onClick={() => setPage(tab.id)}
              className={cn(
                "flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 transition-colors -mb-px cursor-pointer",
                active
                  ? tab.activeColors
                  : "border-transparent text-muted-foreground hover:text-foreground"
              )}
            >
              <Icon className="size-3.5" />
              {tab.label}
              {running && (
                <span
                  className={cn(
                    "size-1.5 rounded-full animate-pulse",
                    tab.dotColor
                  )}
                />
              )}
            </button>
          );
        })}
      </nav>

      {/* ── Content ── */}
      <main className="flex-1 min-h-0">
        {page === "dashboard" && (
          <div className="h-full animate-fade-in">
            <Dashboard
              status={status}
              logs={logs}
              windowInfo={windowInfo}
              onClearLogs={clearLogs}
            />
          </div>
        )}
        {page === "muscu" && (
          <div className="h-full animate-fade-in">
            <Muscu
              muscuStatus={muscuStatus}
              muscuLogs={muscuLogs}
              onClearLogs={clearMuscuLogs}
            />
          </div>
        )}
        {page === "mining" && (
          <div className="h-full animate-fade-in">
            <Mining
              miningStatus={miningStatus}
              miningLogs={miningLogs}
              onClearLogs={clearMiningLogs}
            />
          </div>
        )}
        {page === "settings" && (
          <div className="h-full animate-fade-in">
            <Settings />
          </div>
        )}
        {page === "debug" && (
          <div className="h-full animate-fade-in">
            <DebugSteps logs={logs} onClearLogs={clearLogs} />
          </div>
        )}
      </main>
    </div>
  );
}
