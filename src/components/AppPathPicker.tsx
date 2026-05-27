import { useEffect, useMemo, useState } from "react";
import { AppWindow, ChevronDown, FolderOpen, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  browseAppPath,
  listInstalledApps,
  resolveLaunchPath,
  runningInTauri,
  slugifyAppKey,
  type InstalledApp,
} from "@/lib/tauri";

interface AppPathPickerProps {
  id?: string;
  value: string;
  onChange: (path: string) => void;
  onNamePick?: (key: string) => void;
  onDisplayNamePick?: (name: string) => void;
}

export function AppPathPicker({ id, value, onChange, onNamePick, onDisplayNamePick }: AppPathPickerProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [apps, setApps] = useState<InstalledApp[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open || !runningInTauri()) {
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError(null);

    listInstalledApps()
      .then((items) => {
        if (!cancelled) {
          setApps(items);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(String(err));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [open]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) {
      return apps;
    }
    return apps.filter(
      (app) =>
        app.name.toLowerCase().includes(q) || app.path.toLowerCase().includes(q),
    );
  }, [apps, query]);

  const handleSelect = async (app: InstalledApp) => {
    try {
      const path = await resolveLaunchPath(app.path);
      onChange(path);
      onNamePick?.(slugifyAppKey(app.name));
      onDisplayNamePick?.(app.name);
      setOpen(false);
      setQuery("");
    } catch (err) {
      setError(String(err));
    }
  };

  const handleBrowse = async () => {
    try {
      const path = await browseAppPath();
      if (!path) {
        return;
      }
      onChange(path);
      const baseName = path.split(/[/\\]/).pop() ?? path;
      const displayName = baseName.replace(/\.(app|exe)$/i, "");
      onNamePick?.(slugifyAppKey(displayName));
      onDisplayNamePick?.(displayName);
      setOpen(false);
    } catch (err) {
      setError(String(err));
    }
  };

  if (!runningInTauri()) {
    return (
      <Input
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Nhập path thủ công (mở qua TrayLink app để chọn danh sách)"
      />
    );
  }

  return (
    <div className="space-y-2">
      <div className="flex gap-2">
        <Input
          id={id}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="Chọn app từ danh sách hoặc duyệt file"
          className="font-mono text-xs"
        />
        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger asChild>
            <Button type="button" variant="outline" className="shrink-0">
              <ChevronDown className="mr-1 h-4 w-4" />
              Chọn app
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-[420px] p-0" align="end">
            <div className="border-b p-3">
              <Label htmlFor="app-search" className="sr-only">
                Tìm app
              </Label>
              <div className="relative">
                <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                  id="app-search"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="Tìm theo tên hoặc path..."
                  className="pl-8"
                  autoFocus
                />
              </div>
            </div>

            <ScrollArea className="h-64">
              {loading ? (
                <p className="p-4 text-sm text-muted-foreground">Đang tải danh sách app...</p>
              ) : error ? (
                <p className="p-4 text-sm text-destructive">{error}</p>
              ) : filtered.length === 0 ? (
                <p className="p-4 text-sm text-muted-foreground">Không tìm thấy app nào</p>
              ) : (
                <div className="p-1">
                  {filtered.map((app) => (
                    <button
                      key={`${app.name}-${app.path}`}
                      type="button"
                      onClick={() => handleSelect(app)}
                      className="flex w-full items-center gap-3 rounded-md px-3 py-2 text-left hover:bg-accent"
                    >
                      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md border bg-muted">
                        <AppWindow className="h-5 w-5 text-muted-foreground" />
                      </div>
                      <div className="min-w-0 flex-1">
                        <span className="block text-sm font-medium">{app.name}</span>
                        <span className="block truncate text-xs text-muted-foreground">
                          {app.path}
                        </span>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </ScrollArea>

            <div className="border-t p-2">
              <Button
                type="button"
                variant="secondary"
                className="w-full justify-start"
                onClick={handleBrowse}
              >
                <FolderOpen className="mr-2 h-4 w-4" />
                Duyệt file khác...
              </Button>
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>
  );
}
