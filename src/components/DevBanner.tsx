import { isTauri } from "@tauri-apps/api/core";
import { Badge } from "@/components/ui/badge";

export function DevBanner() {
  if (isTauri()) {
    return null;
  }

  return (
    <div className="border-b border-amber-500/40 bg-amber-500/10 px-6 py-3 text-sm">
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant="outline">Browser mode</Badge>
        <span>
          Bạn đang mở <strong>localhost:1420</strong> trên trình duyệt — Tauri API không
          hoạt động ở đây. Hãy click icon <strong>TrayLink</strong> trên menu bar →{" "}
          <strong>Open Dashboard</strong>.
        </span>
      </div>
      <p className="mt-1 text-muted-foreground">
        Port <strong>1420</strong> = giao diện (Vite). Port <strong>8765</strong> = HTTP API
        launcher (curl / Stream Deck gọi port này).
      </p>
    </div>
  );
}
