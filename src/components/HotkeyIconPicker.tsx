import { useState } from "react";
import { ImagePlus, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import { HotkeyIcon } from "@/components/HotkeyIcon";
import {
  faClassName,
  faIconRef,
  HOTKEY_FA_ICONS,
  parseHotkeyIcon,
} from "@/lib/hotkey-icons";
import { browseDeckIconPath } from "@/lib/tauri";
import { convertFileSrc } from "@tauri-apps/api/core";

interface HotkeyIconPickerProps {
  icon: string | null;
  pendingCustomPath: string | null;
  appKey?: string;
  hotkeyId?: string;
  disabled?: boolean;
  onIconChange: (icon: string | null) => void;
  onPendingCustomPathChange: (path: string | null) => void;
}

export function HotkeyIconPicker({
  icon,
  pendingCustomPath,
  appKey,
  hotkeyId,
  disabled = false,
  onIconChange,
  onPendingCustomPathChange,
}: HotkeyIconPickerProps) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const parsed = parseHotkeyIcon(icon);
  const previewIcon =
    pendingCustomPath && !icon?.startsWith("custom:")
      ? `custom:pending`
      : icon;

  const filteredIcons = HOTKEY_FA_ICONS.filter((item) => {
    const q = search.trim().toLowerCase();
    if (!q) return true;
    return item.id.includes(q) || item.label.toLowerCase().includes(q);
  });

  const handlePickFile = async () => {
    const sourcePath = await browseDeckIconPath();
    if (!sourcePath) return;
    onPendingCustomPathChange(sourcePath);
    onIconChange(null);
    setOpen(false);
  };

  const handleSelectFa = (id: string) => {
    onPendingCustomPathChange(null);
    onIconChange(faIconRef(id));
    setOpen(false);
  };

  const handleClear = () => {
    onPendingCustomPathChange(null);
    onIconChange(null);
  };

  return (
    <div className="space-y-2">
      <Label>Icon</Label>
      <div className="flex items-center gap-2">
        {pendingCustomPath ? (
          <img
            src={convertFileSrc(pendingCustomPath)}
            alt="Icon tùy chỉnh"
            className="h-10 w-10 rounded-lg border bg-background object-contain p-0.5"
          />
        ) : (
          <HotkeyIcon
            icon={previewIcon}
            appKey={appKey}
            hotkeyId={hotkeyId}
            size="md"
          />
        )}

        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger asChild>
            <Button type="button" variant="outline" size="sm" disabled={disabled}>
              Chọn icon
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-80 p-3" align="start">
            <PopoverHeader className="mb-2">
              <PopoverTitle>Font Awesome</PopoverTitle>
              <PopoverDescription>Chọn icon có sẵn hoặc tải hình lên.</PopoverDescription>
            </PopoverHeader>

            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Tìm Save, Copy, Export…"
              className="mb-2 h-8"
            />

            <ScrollArea className="h-48 pr-2">
              <div className="grid grid-cols-6 gap-1">
                {filteredIcons.map((item) => {
                  const selected = parsed.type === "fa" && parsed.value === item.id;
                  return (
                    <button
                      key={item.id}
                      type="button"
                      title={item.label}
                      className={`flex h-9 w-9 items-center justify-center rounded-md border text-sm transition-colors hover:bg-accent ${
                        selected ? "border-primary bg-primary/10" : "border-border/60"
                      }`}
                      onClick={() => handleSelectFa(item.id)}
                    >
                      <i className={faClassName(item.id)} aria-hidden />
                    </button>
                  );
                })}
              </div>
            </ScrollArea>

            <div className="mt-3 flex gap-2">
              <Button type="button" variant="secondary" size="sm" className="flex-1" onClick={() => void handlePickFile()}>
                <ImagePlus className="mr-1 size-4" />
                Tải hình
              </Button>
            </div>
          </PopoverContent>
        </Popover>

        {(icon || pendingCustomPath) && (
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            disabled={disabled}
            onClick={handleClear}
            title="Xóa icon"
          >
            <X className="size-4" />
          </Button>
        )}
      </div>
    </div>
  );
}
