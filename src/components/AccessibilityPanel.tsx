import { useEffect, useState } from "react";
import { CheckCircle2, AlertTriangle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  getAccessibilityStatus,
  openAccessibilitySettings,
  openDevAppBundle,
  promptAccessibilityPermission,
  resetAccessibility,
  signDevBinary,
  testHotkeyInput,
  type AccessibilityStatus,
} from "@/lib/tauri";

export function AccessibilityPanel() {
  const [status, setStatus] = useState<AccessibilityStatus | null>(null);
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);

  const load = async () => {
    setStatus(await getAccessibilityStatus());
  };

  useEffect(() => {
    void load();

    const onVisible = () => {
      if (document.visibilityState === "visible") {
        void load();
      }
    };
    document.addEventListener("visibilitychange", onVisible);
    return () => document.removeEventListener("visibilitychange", onVisible);
  }, []);

  if (!status?.supported) {
    return null;
  }

  const run = async (fn: () => Promise<string>, fallback?: string) => {
    setBusy(true);
    setMessage("");
    try {
      setMessage(await fn());
      await load();
    } catch (err) {
      setMessage(fallback ?? String(err));
    } finally {
      setBusy(false);
    }
  };

  const actionButtons = (
    <div className="flex flex-wrap gap-2">
      <Button variant="outline" onClick={() => void run(async () => {
        const trusted = await promptAccessibilityPermission();
        await load();
        return trusted ? "Đã có quyền Accessibility." : "macOS đã hiện hộp thoại — bật quyền cho TrayLink Dev.app.";
      })} disabled={busy}>
        Yêu cầu quyền
      </Button>
      <Button variant="outline" onClick={() => void openAccessibilitySettings().then(() => setMessage("Đã mở System Settings."))} disabled={busy}>
        Mở cài đặt
      </Button>
      {status.dev_build && (
        <>
          <Button variant="secondary" onClick={() => void run(signDevBinary)} disabled={busy}>
            Tạo / ký Dev.app
          </Button>
          <Button variant="secondary" onClick={() => void run(openDevAppBundle)} disabled={busy}>
            Mở TrayLink Dev.app
          </Button>
          <Button variant="ghost" size="sm" onClick={() => void run(resetAccessibility)} disabled={busy}>
            Reset quyền
          </Button>
        </>
      )}
      <Button variant="outline" onClick={() => void run(testHotkeyInput)} disabled={busy}>
        Thử gửi phím
      </Button>
      <Button variant="ghost" size="sm" onClick={() => void load()} disabled={busy}>
        Kiểm tra lại
      </Button>
    </div>
  );

  return (
    <Card className={status.trusted ? "" : "border-amber-500/40"}>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between gap-3">
          <CardTitle className="text-base">Accessibility (macOS)</CardTitle>
          {status.trusted ? (
            <Badge variant="secondary" className="gap-1 text-emerald-700 dark:text-emerald-400">
              <CheckCircle2 className="size-3.5" />
              Đã bật
            </Badge>
          ) : (
            <Badge variant="destructive" className="gap-1">
              <AlertTriangle className="size-3.5" />
              Chưa bật
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground whitespace-pre-line">{status.hint}</p>

        {status.dev_build && (
          <div className="rounded-md border border-amber-500/30 bg-amber-500/5 p-3 space-y-2 text-xs">
            <p className="text-muted-foreground font-medium">Dev: dùng app bundle (quan trọng)</p>
            {status.dev_app_path ? (
              <code className="block break-all">{status.dev_app_path}</code>
            ) : (
              <p className="text-muted-foreground">Chưa có TrayLink Dev.app — bấm &quot;Tạo / ký Dev.app&quot;</p>
            )}
            <p className="text-muted-foreground">
              Terminal 1: <code className="rounded bg-muted px-1">npm run dev</code> · Terminal 2:{" "}
              <code className="rounded bg-muted px-1">npm run dev:app</code>
            </p>
          </div>
        )}

        {actionButtons}

        {message && <p className="text-sm text-muted-foreground whitespace-pre-line">{message}</p>}
      </CardContent>
    </Card>
  );
}
