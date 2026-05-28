import { useEffect, useState } from "react";
import { Pencil, Trash2 } from "lucide-react";
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { AppIcon } from "@/components/AppIcon";
import { HotkeyIcon } from "@/components/HotkeyIcon";
import { HotkeyIconPicker } from "@/components/HotkeyIconPicker";
import { HotkeyRecorder } from "@/components/HotkeyRecorder";
import { formatAcceleratorDisplay, isValidAccelerator } from "@/lib/hotkeys";
import { appDisplayLabel } from "@/lib/remote-deck";
import {
  cleanupHotkeyIcon,
  setHotkeyIconFromFile,
  slugifyHotkeyId,
  type AppConfig,
  type AppEntry,
  type AppHotkeyBinding,
} from "@/lib/tauri";

interface AppHotkeyDialogProps {
  appKey: string;
  entry: AppEntry;
  config: AppConfig;
  open: boolean;
  saving?: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (updated: AppEntry) => Promise<void>;
}

function resetForm(setters: {
  setName: (v: string) => void;
  setAccelerator: (v: string) => void;
  setAction: (v: "open" | "keys") => void;
  setIcon: (v: string | null) => void;
  setPendingCustomPath: (v: string | null) => void;
  setEditingId: (v: string | null) => void;
  setError: (v: string) => void;
}) {
  setters.setName("");
  setters.setAccelerator("");
  setters.setAction("keys");
  setters.setIcon(null);
  setters.setPendingCustomPath(null);
  setters.setEditingId(null);
  setters.setError("");
}

function findDuplicateAccelerator(
  config: AppConfig,
  accelerator: string,
  excludeAppKey: string,
  excludeHotkeyId?: string,
  action?: "open" | "keys",
): string | null {
  for (const [appKey, app] of Object.entries(config.apps)) {
    for (const binding of app.hotkeys ?? []) {
      if (appKey === excludeAppKey && binding.id === excludeHotkeyId) {
        continue;
      }
      if (binding.accelerator === accelerator) {
        if (appKey === excludeAppKey) {
          return `${appKey} → ${binding.name}`;
        }
        if (action === "open" && binding.action === "open") {
          return `${appKey} → ${binding.name} (phím tắt toàn hệ thống)`;
        }
      }
    }
  }
  return null;
}

function uniqueHotkeyId(existing: AppHotkeyBinding[], base: string): string {
  if (!existing.some((item) => item.id === base)) {
    return base;
  }
  let index = 2;
  while (existing.some((item) => item.id === `${base}-${index}`)) {
    index += 1;
  }
  return `${base}-${index}`;
}

export function AppHotkeyDialog({
  appKey,
  entry,
  config,
  open,
  saving = false,
  onOpenChange,
  onSave,
}: AppHotkeyDialogProps) {
  const [hotkeys, setHotkeys] = useState<AppHotkeyBinding[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [accelerator, setAccelerator] = useState("");
  const [action, setAction] = useState<"open" | "keys">("keys");
  const [icon, setIcon] = useState<string | null>(null);
  const [pendingCustomPath, setPendingCustomPath] = useState<string | null>(null);
  const [error, setError] = useState("");

  const displayName = entry.name ?? appDisplayLabel(appKey, entry.path, entry.name);
  const isEditing = editingId !== null;

  useEffect(() => {
    if (!open) return;
    setHotkeys(entry.hotkeys ?? []);
    resetForm({
      setName,
      setAccelerator,
      setAction,
      setIcon,
      setPendingCustomPath,
      setEditingId,
      setError,
    });
  }, [open, entry.hotkeys]);

  const persist = async (nextHotkeys: AppHotkeyBinding[]) => {
    await onSave({
      ...entry,
      hotkeys: nextHotkeys,
    });
    setHotkeys(nextHotkeys);
  };

  const cleanupIfCustom = async (iconValue?: string | null) => {
    if (iconValue?.startsWith("custom:")) {
      await cleanupHotkeyIcon(iconValue);
    }
  };

  const validateAndBuildBinding = (): AppHotkeyBinding | null => {
    const trimmedName = name.trim();
    if (!trimmedName || !accelerator) {
      setError("Vui lòng nhập tên và ghi phím tắt");
      return null;
    }

    const duplicate = findDuplicateAccelerator(config, accelerator, appKey, editingId ?? undefined, action);
    if (duplicate) {
      setError(`Phím tắt này đã được dùng bởi ${duplicate}`);
      return null;
    }

    if (!isValidAccelerator(accelerator)) {
      setError("Phím tắt không hợp lệ — dùng phím A-Z, 0-9 hoặc phím đặc biệt (Space, Delete…)");
      return null;
    }

    return {
      id: editingId ?? uniqueHotkeyId(hotkeys, slugifyHotkeyId(trimmedName)),
      name: trimmedName,
      accelerator,
      action,
    };
  };

  const handleSubmit = async () => {
    const binding = validateAndBuildBinding();
    if (!binding) return;

    const previous = editingId ? hotkeys.find((item) => item.id === editingId) : null;
    let nextBinding: AppHotkeyBinding = {
      ...binding,
      icon: icon ?? undefined,
    };

    let next = isEditing
      ? hotkeys.map((item) => (item.id === editingId ? nextBinding : item))
      : [...hotkeys, nextBinding];

    setError("");

    if (previous?.icon && previous.icon !== nextBinding.icon) {
      await cleanupIfCustom(previous.icon);
    }

    await persist(next);

    if (pendingCustomPath) {
      const iconRef = await setHotkeyIconFromFile(appKey, nextBinding.id, pendingCustomPath);
      nextBinding = { ...nextBinding, icon: iconRef };
      next = next.map((item) => (item.id === nextBinding.id ? nextBinding : item));
      await persist(next);
    }

    resetForm({
      setName,
      setAccelerator,
      setAction,
      setIcon,
      setPendingCustomPath,
      setEditingId,
      setError,
    });
  };

  const handleEdit = (binding: AppHotkeyBinding) => {
    setEditingId(binding.id);
    setName(binding.name);
    setAccelerator(binding.accelerator);
    setAction(binding.action === "open" ? "open" : "keys");
    setIcon(binding.icon ?? null);
    setPendingCustomPath(null);
    setError("");
  };

  const handleCancelEdit = () => {
    resetForm({
      setName,
      setAccelerator,
      setAction,
      setIcon,
      setPendingCustomPath,
      setEditingId,
      setError,
    });
  };

  const handleRemove = async (binding: AppHotkeyBinding) => {
    setError("");
    if (editingId === binding.id) {
      handleCancelEdit();
    }
    await cleanupIfCustom(binding.icon);
    await persist(hotkeys.filter((item) => item.id !== binding.id));
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Phím tắt — {displayName}</DialogTitle>
          <DialogDescription>
            Phím <strong>Gửi phím</strong> (Save, Copy…) hoạt động trên Remote Deck — không chặn phím
            tắt gốc của app. Phím <strong>Mở / focus app</strong> có thể gán tổ hợp toàn hệ thống.
            Cần quyền Accessibility trên macOS.
          </DialogDescription>
        </DialogHeader>

        <div className="flex items-center gap-3 rounded-lg border border-border/60 bg-muted/30 p-3">
          <AppIcon path={entry.path} name={displayName} size="lg" />
          <div className="min-w-0">
            <p className="font-medium truncate">{displayName}</p>
            <p className="text-xs text-muted-foreground font-mono truncate">{appKey}</p>
          </div>
        </div>

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-12">Icon</TableHead>
              <TableHead>Tên</TableHead>
              <TableHead>Phím tắt</TableHead>
              <TableHead>Hành động</TableHead>
              <TableHead className="text-right w-24">Thao tác</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {hotkeys.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  Chưa có phím tắt — thêm Save, Export, Copy bên dưới
                </TableCell>
              </TableRow>
            ) : (
              hotkeys.map((binding) => (
                <TableRow key={binding.id} className={editingId === binding.id ? "bg-muted/40" : ""}>
                  <TableCell>
                    <HotkeyIcon
                      icon={binding.icon}
                      appKey={appKey}
                      hotkeyId={binding.id}
                      name={binding.name}
                    />
                  </TableCell>
                  <TableCell className="font-medium">{binding.name}</TableCell>
                  <TableCell className="font-mono text-xs">
                    {formatAcceleratorDisplay(binding.accelerator)}
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {binding.action === "open" ? "Mở app" : "Gửi phím"}
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      <Button
                        variant="ghost"
                        size="sm"
                        disabled={saving}
                        onClick={() => handleEdit(binding)}
                        title="Sửa"
                      >
                        <Pencil className="size-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="text-destructive hover:text-destructive"
                        disabled={saving}
                        onClick={() => void handleRemove(binding)}
                        title="Xóa"
                      >
                        <Trash2 className="size-4" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>

        <div className="rounded-lg border border-border/60 p-3 space-y-3">
          <p className="text-sm font-medium">{isEditing ? "Sửa phím tắt" : "Thêm phím tắt mới"}</p>

          <div className="grid gap-3 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="hotkey-name">Tên (vd. Save, Export, Copy)</Label>
              <Input
                id="hotkey-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Save"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="hotkey-action">Loại hành động</Label>
              <select
                id="hotkey-action"
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs"
                value={action}
                onChange={(e) => setAction(e.target.value as "open" | "keys")}
              >
                <option value="keys">Gửi phím tắt vào app</option>
                <option value="open">Mở / focus app</option>
              </select>
            </div>
          </div>

          <div className="space-y-2">
            <Label>Phím tắt</Label>
            <HotkeyRecorder value={accelerator} onChange={setAccelerator} disabled={saving} />
          </div>

          <HotkeyIconPicker
            icon={icon}
            pendingCustomPath={pendingCustomPath}
            appKey={appKey}
            hotkeyId={editingId ?? undefined}
            disabled={saving}
            onIconChange={setIcon}
            onPendingCustomPathChange={setPendingCustomPath}
          />
        </div>

        {error && <p className="text-sm text-destructive">{error}</p>}

        <DialogFooter className="gap-2 sm:gap-0">
          {isEditing && (
            <Button variant="ghost" onClick={handleCancelEdit} disabled={saving}>
              Hủy sửa
            </Button>
          )}
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>
            Đóng
          </Button>
          <Button
            onClick={() => void handleSubmit()}
            disabled={saving || !name.trim() || !accelerator}
          >
            {saving ? "Đang lưu..." : isEditing ? "Lưu thay đổi" : "Thêm phím tắt"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
