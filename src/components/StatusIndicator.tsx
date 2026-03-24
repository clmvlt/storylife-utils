import type { AutomationStatus } from "@/lib/invoke";
import { cn } from "@/lib/utils";
import {
  Check,
  Loader2,
  Power,
  WifiOff,
  Search,
  Gamepad2,
  Keyboard,
  Zap,
} from "lucide-react";
import type { ReactNode } from "react";

interface Props {
  status: AutomationStatus;
}

interface StateConfig {
  color: string;
  label: string;
  icon: ReactNode;
  badgeLabel: string;
}

const STATE_MAP: Record<AutomationStatus["state"], StateConfig> = {
  idle: {
    color: "from-zinc-600 to-zinc-700",
    label: "En attente",
    icon: <Power className="size-8 text-white/70" />,
    badgeLabel: "Idle",
  },
  searching_window: {
    color: "from-blue-500 to-blue-600",
    label: "Recherche de FiveM...",
    icon: <Search className="size-8 text-white" />,
    badgeLabel: "Recherche",
  },
  launching_fivem: {
    color: "from-amber-500 to-orange-500",
    label: "Lancement de FiveM...",
    icon: <Gamepad2 className="size-8 text-white" />,
    badgeLabel: "Lancement",
  },
  waiting_ocr: {
    color: "from-orange-500 to-orange-600",
    label: "Détection du personnage...",
    icon: <Loader2 className="size-8 text-white animate-spin" />,
    badgeLabel: "OCR",
  },
  selecting_character: {
    color: "from-purple-500 to-violet-600",
    label: "Sélection du personnage...",
    icon: <Zap className="size-8 text-white" />,
    badgeLabel: "Sélection",
  },
  sending_keys: {
    color: "from-violet-500 to-purple-600",
    label: "Envoi des touches...",
    icon: <Keyboard className="size-8 text-white" />,
    badgeLabel: "Touches",
  },
  afk_active: {
    color: "from-emerald-500 to-green-600",
    label: "Mode AFK actif",
    icon: <Check className="size-9 text-white" strokeWidth={3} />,
    badgeLabel: "AFK Actif",
  },
  reconnecting: {
    color: "from-red-500 to-red-600",
    label: "Reconnexion...",
    icon: <WifiOff className="size-8 text-white" />,
    badgeLabel: "Reconnexion",
  },
};

export default function StatusIndicator({ status }: Props) {
  const cfg = STATE_MAP[status.state];
  const active = status.running;

  return (
    <div className="flex flex-col items-center gap-6">
      {/* Orb */}
      <div className="relative flex items-center justify-center size-40">
        {active && (
          <>
            <div
              className={cn(
                "absolute inset-0 rounded-full bg-gradient-to-br opacity-20 animate-pulse-glow",
                cfg.color
              )}
            />
            <div
              className={cn(
                "absolute inset-5 rounded-full bg-gradient-to-br opacity-25 animate-pulse-glow",
                cfg.color
              )}
              style={{ animationDelay: "0.6s" }}
            />
          </>
        )}
        <div
          className={cn(
            "relative size-[4.5rem] rounded-full bg-gradient-to-br flex items-center justify-center shadow-xl transition-all duration-500",
            cfg.color
          )}
        >
          {cfg.icon}
        </div>
      </div>

      {/* Label */}
      <div className="text-center space-y-2">
        <p className="text-base font-semibold tracking-tight">{cfg.label}</p>
        {status.afk_start_time && status.state === "afk_active" && (
          <p className="text-xs text-muted-foreground">
            Depuis {status.afk_start_time}
          </p>
        )}
      </div>
    </div>
  );
}
