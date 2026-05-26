import { useEffect, useState } from "react";
import { AppWindow } from "lucide-react";
import { cn } from "@/lib/utils";
import { getAppIconCached, peekAppIconCache } from "@/lib/icon-cache";

interface AppIconProps {
  path: string;
  name: string;
  size?: "sm" | "md" | "lg";
  className?: string;
}

const sizeClasses = {
  sm: { box: "h-8 w-8 min-h-8 min-w-8", icon: "h-4 w-4", pad: "p-0.5" },
  md: { box: "h-10 w-10 min-h-10 min-w-10", icon: "h-5 w-5", pad: "p-1" },
  lg: { box: "h-12 w-12 min-h-12 min-w-12", icon: "h-6 w-6", pad: "p-1" },
} as const;

export function AppIcon({ path, name, size = "sm", className }: AppIconProps) {
  const [iconUrl, setIconUrl] = useState<string | null>(() => peekAppIconCache(path) ?? null);
  const styles = sizeClasses[size];

  useEffect(() => {
    let cancelled = false;
    const cached = peekAppIconCache(path);
    if (cached !== undefined) {
      setIconUrl(cached);
      return;
    }

    getAppIconCached(path)
      .then((url) => {
        if (!cancelled) {
          setIconUrl(url);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setIconUrl(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [path]);

  if (iconUrl) {
    return (
      <img
        src={iconUrl}
        alt={name}
        className={cn(
          "shrink-0 rounded-md border bg-background object-contain",
          styles.box,
          styles.pad,
          className,
        )}
      />
    );
  }

  return (
    <div
      className={cn(
        "flex shrink-0 items-center justify-center rounded-md border bg-muted",
        styles.box,
        className,
      )}
    >
      <AppWindow className={cn(styles.icon, "text-muted-foreground")} />
    </div>
  );
}
