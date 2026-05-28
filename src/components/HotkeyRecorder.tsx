import { useEffect, useState } from "react";
import { Keyboard } from "lucide-react";
import { Button } from "@/components/ui/button";
import { formatAcceleratorDisplay, keyEventToAccelerator } from "@/lib/hotkeys";

interface HotkeyRecorderProps {
  value: string;
  onChange: (accelerator: string) => void;
  disabled?: boolean;
}

export function HotkeyRecorder({ value, onChange, disabled }: HotkeyRecorderProps) {
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    if (!recording) {
      return;
    }

    const handler = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      if (event.key === "Escape") {
        setRecording(false);
        return;
      }

      const accelerator = keyEventToAccelerator(event);
      if (accelerator) {
        onChange(accelerator);
        setRecording(false);
      }
    };

    window.addEventListener("keydown", handler, true);
    return () => window.removeEventListener("keydown", handler, true);
  }, [recording, onChange]);

  return (
    <Button
      type="button"
      variant={recording ? "default" : "outline"}
      className="min-w-[180px] justify-start font-mono"
      disabled={disabled}
      onClick={() => setRecording(true)}
    >
      <Keyboard className="mr-2 size-4 shrink-0" />
      {recording
        ? "Nhấn tổ hợp phím… (Esc hủy)"
        : value
          ? formatAcceleratorDisplay(value)
          : "Ghi phím tắt"}
    </Button>
  );
}
