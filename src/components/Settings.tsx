import { useState, useEffect, useCallback } from "react";
import {
  getConfig,
  saveConfig,
  getWebhookConfig,
  saveWebhookConfig,
  testWebhook,
  getStats,
  resetStats,
  type Config,
  type WebhookConfig,
  type Stats,
} from "@/lib/invoke";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
  Save,
  CheckCircle2,
  User,
  Info,
  Bell,
  BarChart3,
  Send,
  Loader2,
  Trash2,
  Dumbbell,
  Gem,
  Clock,
} from "lucide-react";
import { cn } from "@/lib/utils";

function formatDuration(totalSeconds: number): string {
  if (totalSeconds <= 0) return "0m";
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  if (days > 0) return `${days}j ${hours}h ${minutes}m`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

function formatNumber(n: number): string {
  return n.toLocaleString("fr-FR");
}

function Toggle({
  enabled,
  onChange,
  label,
}: {
  enabled: boolean;
  onChange: (v: boolean) => void;
  label: string;
}) {
  return (
    <button
      type="button"
      onClick={() => onChange(!enabled)}
      className="flex items-center gap-2.5 w-full py-1 cursor-pointer group"
    >
      <div
        className={cn(
          "relative w-8 h-[18px] rounded-full transition-colors shrink-0",
          enabled ? "bg-primary" : "bg-zinc-700"
        )}
      >
        <div
          className={cn(
            "absolute top-[2px] size-[14px] rounded-full bg-white transition-transform",
            enabled ? "translate-x-[15px]" : "translate-x-[2px]"
          )}
        />
      </div>
      <span className="text-xs text-muted-foreground group-hover:text-foreground transition-colors">
        {label}
      </span>
    </button>
  );
}

export default function Settings() {
  const [config, setConfig] = useState<Config>({ character_name: "" });
  const [saved, setSaved] = useState(false);
  const [loading, setLoading] = useState(true);

  // Webhook
  const [webhook, setWebhook] = useState<WebhookConfig>({
    url: "",
    notify_afk_start: true,
    notify_afk_stop: true,
    notify_crash: true,
    notify_muscu_start: true,
    notify_muscu_stop: true,
    notify_mining_start: true,
    notify_mining_stop: true,
  });
  const [webhookSaved, setWebhookSaved] = useState(false);
  const [webhookTesting, setWebhookTesting] = useState(false);
  const [webhookTestResult, setWebhookTestResult] = useState<{
    ok: boolean;
    msg: string;
  } | null>(null);

  // Stats
  const [stats, setStats] = useState<Stats>({
    total_muscu_cycles: 0,
    total_mining_clicks: 0,
    total_afk_seconds: 0,
  });

  useEffect(() => {
    Promise.all([getConfig(), getWebhookConfig(), getStats()])
      .then(([c, w, s]) => {
        setConfig(c);
        setWebhook(w);
        setStats(s);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  // Refresh stats every 10s
  useEffect(() => {
    const interval = setInterval(() => {
      getStats().then(setStats).catch(() => {});
    }, 10_000);
    return () => clearInterval(interval);
  }, []);

  const handleSave = useCallback(async () => {
    await saveConfig(config);
    setSaved(true);
    setTimeout(() => setSaved(false), 2500);
  }, [config]);

  const handleWebhookSave = useCallback(async () => {
    await saveWebhookConfig(webhook);
    setWebhookSaved(true);
    setTimeout(() => setWebhookSaved(false), 2500);
  }, [webhook]);

  const handleWebhookTest = useCallback(async () => {
    if (!webhook.url.trim()) return;
    setWebhookTesting(true);
    setWebhookTestResult(null);
    try {
      await testWebhook(webhook.url);
      setWebhookTestResult({ ok: true, msg: "Envoyé !" });
    } catch (e) {
      setWebhookTestResult({ ok: false, msg: String(e) });
    } finally {
      setWebhookTesting(false);
      setTimeout(() => setWebhookTestResult(null), 4000);
    }
  }, [webhook.url]);

  const handleResetStats = useCallback(async () => {
    await resetStats();
    setStats({ total_muscu_cycles: 0, total_mining_clicks: 0, total_afk_seconds: 0 });
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="size-6 border-2 border-primary border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-6">
      <div className="max-w-md mx-auto space-y-5">
        {/* Character */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <User className="size-4 text-primary" />
              Personnage
            </CardTitle>
            <CardDescription>
              Nom tel qu'il apparaît sur l'écran de sélection FiveM. L'OCR
              recherchera ce texte.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="name">Nom du personnage</Label>
              <Input
                id="name"
                value={config.character_name}
                onChange={(e) =>
                  setConfig({ ...config, character_name: e.target.value })
                }
                placeholder="Ex : William Reed"
                className="h-9"
              />
            </div>
            <Button onClick={handleSave} size="sm">
              {saved ? (
                <>
                  <CheckCircle2 />
                  Sauvegardé
                </>
              ) : (
                <>
                  <Save />
                  Sauvegarder
                </>
              )}
            </Button>
          </CardContent>
        </Card>

        {/* Discord Webhook */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Bell className="size-4 text-[#5865F2]" />
              Webhook Discord
            </CardTitle>
            <CardDescription>
              Recevez des notifications Discord quand un événement se produit.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* URL */}
            <div className="space-y-2">
              <Label htmlFor="webhook-url">URL du webhook</Label>
              <Input
                id="webhook-url"
                value={webhook.url}
                onChange={(e) =>
                  setWebhook({ ...webhook, url: e.target.value })
                }
                placeholder="https://discord.com/api/webhooks/..."
                className="h-9 font-mono text-xs"
              />
            </div>

            {/* Test + Save buttons */}
            <div className="flex gap-2">
              <Button
                onClick={handleWebhookTest}
                variant="secondary"
                size="sm"
                disabled={!webhook.url.trim() || webhookTesting}
                className="gap-1.5"
              >
                {webhookTesting ? (
                  <Loader2 className="size-3.5 animate-spin" />
                ) : (
                  <Send className="size-3.5" />
                )}
                Tester
              </Button>
              <Button onClick={handleWebhookSave} size="sm" className="gap-1.5">
                {webhookSaved ? (
                  <>
                    <CheckCircle2 className="size-3.5" />
                    Sauvegardé
                  </>
                ) : (
                  <>
                    <Save className="size-3.5" />
                    Sauvegarder
                  </>
                )}
              </Button>
            </div>

            {/* Test result */}
            {webhookTestResult && (
              <p
                className={cn(
                  "text-[11px]",
                  webhookTestResult.ok
                    ? "text-emerald-400"
                    : "text-destructive"
                )}
              >
                {webhookTestResult.msg}
              </p>
            )}

            <Separator />

            {/* Notification toggles */}
            <div className="space-y-3">
              <p className="text-[10px] uppercase tracking-wider text-muted-foreground font-medium">
                Notifications
              </p>

              {/* AFK */}
              <div className="space-y-0.5">
                <p className="text-[10px] uppercase tracking-wider text-emerald-400/80 font-medium mb-1">
                  AFK
                </p>
                <Toggle
                  enabled={webhook.notify_afk_start}
                  onChange={(v) =>
                    setWebhook({ ...webhook, notify_afk_start: v })
                  }
                  label="AFK démarré"
                />
                <Toggle
                  enabled={webhook.notify_afk_stop}
                  onChange={(v) =>
                    setWebhook({ ...webhook, notify_afk_stop: v })
                  }
                  label="AFK arrêté"
                />
                <Toggle
                  enabled={webhook.notify_crash}
                  onChange={(v) =>
                    setWebhook({ ...webhook, notify_crash: v })
                  }
                  label="Crash détecté"
                />
              </div>

              {/* Muscu */}
              <div className="space-y-0.5">
                <p className="text-[10px] uppercase tracking-wider text-violet-400/80 font-medium mb-1">
                  Muscu
                </p>
                <Toggle
                  enabled={webhook.notify_muscu_start}
                  onChange={(v) =>
                    setWebhook({ ...webhook, notify_muscu_start: v })
                  }
                  label="Muscu démarré"
                />
                <Toggle
                  enabled={webhook.notify_muscu_stop}
                  onChange={(v) =>
                    setWebhook({ ...webhook, notify_muscu_stop: v })
                  }
                  label="Muscu arrêté"
                />
              </div>

              {/* Mining */}
              <div className="space-y-0.5">
                <p className="text-[10px] uppercase tracking-wider text-amber-400/80 font-medium mb-1">
                  Mining
                </p>
                <Toggle
                  enabled={webhook.notify_mining_start}
                  onChange={(v) =>
                    setWebhook({ ...webhook, notify_mining_start: v })
                  }
                  label="Mining démarré"
                />
                <Toggle
                  enabled={webhook.notify_mining_stop}
                  onChange={(v) =>
                    setWebhook({ ...webhook, notify_mining_stop: v })
                  }
                  label="Mining arrêté"
                />
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Stats */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="size-4 text-primary" />
              Statistiques
            </CardTitle>
            <CardDescription>
              Totaux cumulés depuis le début.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="grid gap-2">
              {/* AFK time */}
              <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-secondary/50">
                <Clock className="size-4 text-emerald-400" />
                <div className="flex-1">
                  <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                    Temps AFK total
                  </p>
                  <p className="text-sm font-semibold tabular-nums">
                    {formatDuration(stats.total_afk_seconds)}
                  </p>
                </div>
              </div>

              {/* Muscu cycles */}
              <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-secondary/50">
                <Dumbbell className="size-4 text-violet-400" />
                <div className="flex-1">
                  <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                    Cycles Muscu
                  </p>
                  <p className="text-sm font-semibold tabular-nums">
                    {formatNumber(stats.total_muscu_cycles)}
                  </p>
                </div>
              </div>

              {/* Mining clicks */}
              <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-secondary/50">
                <Gem className="size-4 text-amber-400" />
                <div className="flex-1">
                  <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                    Points de mine cassés
                  </p>
                  <p className="text-sm font-semibold tabular-nums">
                    {formatNumber(stats.total_mining_clicks)}
                  </p>
                </div>
              </div>
            </div>

            <Button
              onClick={handleResetStats}
              variant="ghost"
              size="sm"
              className="gap-1.5 text-muted-foreground hover:text-destructive"
            >
              <Trash2 className="size-3" />
              Réinitialiser
            </Button>
          </CardContent>
        </Card>

        {/* About */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Info className="size-4 text-primary" />À propos
            </CardTitle>
          </CardHeader>
          <CardContent>
            <dl className="space-y-3 text-sm">
              {[
                ["Version", "1.0.1"],
                ["Serveur", "StoryLife"],
                ["URI", "cfx.re/join/aaex7k"],
                ["OCR", "Windows OCR (natif)"],
                ["Auteur", "DIMZOU"],
              ].map(([k, v]) => (
                <div key={k}>
                  <div className="flex items-center justify-between">
                    <dt className="text-muted-foreground">{k}</dt>
                    <dd className="font-medium">{v}</dd>
                  </div>
                  <Separator className="mt-3" />
                </div>
              ))}
            </dl>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
