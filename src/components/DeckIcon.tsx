import { useEffect, useState } from "react";
import { AppWindow, Terminal } from "lucide-react";
import { AppIcon } from "@/components/AppIcon";
import { getDeckIconDataUrl } from "@/lib/tauri";
import type { DeckEditorItem } from "@/lib/remote-deck";

interface DeckIconProps {
  item: DeckEditorItem;
  size?: "sm" | "md";
  className?: string;
}

export function DeckIcon({ item, size = "sm", className = "" }: DeckIconProps) {
  const [customUrl, setCustomUrl] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    if (item.customIcon) {
      getDeckIconDataUrl(item.type, item.key)
        .then((url) => {
          if (!cancelled) setCustomUrl(url);
        })
        .catch(() => {
          if (!cancelled) setCustomUrl(null);
        });
    } else {
      setCustomUrl(null);
    }

    return () => {
      cancelled = true;
    };
  }, [item.customIcon, item.type, item.key]);

  const boxClass =
    size === "md"
      ? "h-12 w-12 rounded-lg"
      : "h-8 w-8 rounded-md";
  const iconClass = size === "md" ? "h-7 w-7" : "h-4 w-4";

  if (customUrl) {
    return (
      <img
        src={customUrl}
        alt={item.label}
        className={`${boxClass} border bg-background object-contain p-0.5 ${className}`}
      />
    );
  }

  if (item.type === "app" && item.path) {
    return <AppIcon path={item.path} name={item.key} />;
  }

  if (item.type === "app") {
    return (
      <div className={`flex ${boxClass} items-center justify-center border bg-muted ${className}`}>
        <AppWindow className={`${iconClass} text-muted-foreground`} />
      </div>
    );
  }

  return (
    <div className={`flex ${boxClass} items-center justify-center border bg-muted ${className}`}>
      <Terminal className={`${iconClass} text-muted-foreground`} />
    </div>
  );
}
