import { useEffect, useState } from "react";
import { AppWindow } from "lucide-react";
import { getAppIcon } from "@/lib/tauri";

interface AppIconProps {
  path: string;
  name: string;
}

export function AppIcon({ path, name }: AppIconProps) {
  const [iconUrl, setIconUrl] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    getAppIcon(path)
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
        className="h-8 w-8 rounded-md border bg-background object-contain p-0.5"
      />
    );
  }

  return (
    <div className="flex h-8 w-8 items-center justify-center rounded-md border bg-muted">
      <AppWindow className="h-4 w-4 text-muted-foreground" />
    </div>
  );
}
