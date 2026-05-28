import { useEffect, useState } from "react";
import { Keyboard } from "lucide-react";
import { faClassName, parseHotkeyIcon } from "@/lib/hotkey-icons";
import { getHotkeyIconDataUrl } from "@/lib/tauri";

interface HotkeyIconProps {
  icon?: string | null;
  appKey?: string;
  hotkeyId?: string;
  name?: string;
  size?: "sm" | "md";
  className?: string;
}

export function HotkeyIcon({
  icon,
  appKey,
  hotkeyId,
  name = "",
  size = "sm",
  className = "",
}: HotkeyIconProps) {
  const [customUrl, setCustomUrl] = useState<string | null>(null);
  const parsed = parseHotkeyIcon(icon);
  const boxClass = size === "md" ? "h-10 w-10 rounded-lg text-lg" : "h-8 w-8 rounded-md text-sm";
  const iconSize = size === "md" ? "text-xl" : "text-base";

  useEffect(() => {
    let cancelled = false;

    if (parsed.type === "custom" && appKey && hotkeyId) {
      getHotkeyIconDataUrl(appKey, hotkeyId)
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
  }, [parsed.type, appKey, hotkeyId, icon]);

  if (parsed.type === "custom" && customUrl) {
    return (
      <img
        src={customUrl}
        alt={name}
        className={`${boxClass} border bg-background object-contain p-0.5 ${className}`}
      />
    );
  }

  if (parsed.type === "fa") {
    return (
      <div
        className={`flex ${boxClass} items-center justify-center border bg-muted ${iconSize} ${className}`}
        aria-hidden
      >
        <i className={faClassName(parsed.value)} />
      </div>
    );
  }

  return (
    <div
      className={`flex ${boxClass} items-center justify-center border bg-muted ${className}`}
      aria-hidden
    >
      <Keyboard className={size === "md" ? "size-5 text-muted-foreground" : "size-4 text-muted-foreground"} />
    </div>
  );
}
