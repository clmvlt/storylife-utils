import { useEffect, useRef } from "react";
import type { LogEntry } from "@/lib/invoke";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Trash2, Terminal } from "lucide-react";
import { cn } from "@/lib/utils";

interface Props {
  logs: LogEntry[];
  onClear: () => void;
}

const DOT: Record<string, string> = {
  info: "bg-blue-400",
  warn: "bg-amber-400",
  error: "bg-red-400",
};

const TEXT: Record<string, string> = {
  info: "text-blue-300/90",
  warn: "text-amber-300/90",
  error: "text-red-300/90",
};

export default function LogViewer({ logs, onClear }: Props) {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  return (
    <div className="flex flex-col h-full rounded-xl border border-border bg-card overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 h-10 border-b border-border bg-secondary/40">
        <div className="flex items-center gap-2 text-muted-foreground">
          <Terminal className="size-3.5" />
          <span className="text-xs font-semibold tracking-wide uppercase">
            Console
          </span>
          <Badge variant="secondary" className="font-mono text-[10px] tabular-nums">
            {logs.length}
          </Badge>
        </div>
        <Button variant="ghost" size="xs" onClick={onClear}>
          <Trash2 className="size-3" />
          Effacer
        </Button>
      </div>

      {/* Logs */}
      <ScrollArea className="flex-1">
        <div className="p-2 font-mono text-[11px] leading-5">
          {logs.length === 0 && (
            <div className="flex flex-col items-center justify-center py-20 text-muted-foreground gap-2">
              <Terminal className="size-7 opacity-20" />
              <p className="text-xs">En attente de logs...</p>
            </div>
          )}
          {logs.map((log, i) => (
            <div
              key={i}
              className="flex items-start gap-2 px-2 py-0.5 rounded-md hover:bg-secondary/40 animate-fade-in"
            >
              <span className="text-muted-foreground/50 shrink-0 w-[3.2rem] tabular-nums select-text text-right">
                {log.timestamp}
              </span>
              <span
                className={cn("size-1.5 rounded-full mt-[7px] shrink-0", DOT[log.level])}
              />
              <span className={cn("select-text break-all", TEXT[log.level])}>
                {log.message}
              </span>
            </div>
          ))}
          <div ref={endRef} />
        </div>
      </ScrollArea>
    </div>
  );
}
