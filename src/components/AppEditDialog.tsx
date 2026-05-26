import { useEffect, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { AppPathPicker } from "@/components/AppPathPicker";
import { isBrowserApp, validateAppUrl } from "@/lib/browser";
import type { AppEntry } from "@/lib/tauri";

interface AppEditDialogProps {
  appKey: string;
  entry: AppEntry;
  open: boolean;
  saving?: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (updated: AppEntry) => Promise<void>;
}

export function AppEditDialog({
  appKey,
  entry,
  open,
  saving = false,
  onOpenChange,
  onSave,
}: AppEditDialogProps) {
  const [name, setName] = useState("");
  const [path, setPath] = useState("");
  const [url, setUrl] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    if (!open) return;
    setName(entry.name ?? "");
    setPath(entry.path);
    setUrl(entry.url ?? "");
    setError("");
  }, [open, entry]);

  const showUrlField = isBrowserApp(path);

  const handlePathChange = (newPath: string) => {
    setPath(newPath);
    if (!isBrowserApp(newPath)) {
      setUrl("");
    }
  };

  const handleSave = async () => {
    if (!path.trim()) {
      setError("Path không được để trống");
      return;
    }

    const urlError = validateAppUrl(url);
    if (urlError) {
      setError(urlError);
      return;
    }

    setError("");
    await onSave({
      ...entry,
      path: path.trim(),
      name: name.trim() || undefined,
      url: showUrlField ? url.trim() || undefined : undefined,
    });
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>Sửa app</DialogTitle>
          <DialogDescription>
            Key <code className="rounded bg-muted px-1">{appKey}</code> — dùng trong API, không đổi được.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor={`edit-name-${appKey}`}>Tên hiển thị</Label>
            <Input
              id={`edit-name-${appKey}`}
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="VD: Google Chrome"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor={`edit-path-${appKey}`}>Path / App</Label>
            <AppPathPicker
              id={`edit-path-${appKey}`}
              value={path}
              onChange={handlePathChange}
              onDisplayNamePick={(pickedName) => {
                if (!name.trim()) {
                  setName(pickedName);
                }
              }}
            />
          </div>

          {showUrlField && (
            <div className="space-y-2">
              <Label htmlFor={`edit-url-${appKey}`}>URL mặc định (tùy chọn)</Label>
              <Input
                id={`edit-url-${appKey}`}
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://example.com"
              />
              <p className="text-xs text-muted-foreground">
                Trình duyệt sẽ mở URL này khi gọi API. Có thể ghi đè bằng{" "}
                <code className="rounded bg-muted px-1">{`{"app":"...","url":"..."}`}</code>
              </p>
            </div>
          )}

          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>
            Hủy
          </Button>
          <Button onClick={() => void handleSave()} disabled={saving}>
            {saving ? "Đang lưu..." : "Lưu"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
