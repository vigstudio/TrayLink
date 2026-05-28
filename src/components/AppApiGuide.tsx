import { useState } from "react";
import { Check, Copy } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { appSupportsUrl } from "@/lib/browser";
import { apiBaseUrl, apiGetUrl, type AppEntry, type AppHotkeyBinding } from "@/lib/tauri";

interface AppApiGuideProps {
  appKey: string;
  entry: AppEntry;
  port: number;
  token: string;
  requireToken: boolean;
  allowGet: boolean;
  lanIp?: string | null;
}

function CopyBlock({ label, text }: { label: string; text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div>
      <div className="mb-1 flex items-center justify-between gap-2">
        <p className="font-medium">{label}</p>
        <Button variant="outline" size="sm" className="h-7 shrink-0 px-2" onClick={handleCopy}>
          {copied ? (
            <>
              <Check className="size-3.5" />
              Đã copy
            </>
          ) : (
            <>
              <Copy className="size-3.5" />
              Copy
            </>
          )}
        </Button>
      </div>
      <pre className="overflow-x-auto whitespace-pre-wrap break-all rounded-md bg-muted p-3 text-xs">
        {text}
      </pre>
    </div>
  );
}

function buildOpenAppCurl(
  appKey: string,
  port: number,
  token: string,
  requireToken: boolean,
  lanIp: string | null | undefined,
  url?: string,
) {
  const body = url
    ? JSON.stringify({ app: appKey, url }, null, 2)
    : JSON.stringify({ app: appKey }, null, 2);

  const authHeader = requireToken
    ? `  -H "Authorization: Bearer ${token}" \\\n`
    : "";

  return `curl -X POST ${apiBaseUrl(port, lanIp)}/open-app \\
${authHeader}  -H "Content-Type: application/json" \\
  -d '${body.replace(/'/g, "'\\''")}'`;
}

function buildSendHotkeyCurl(
  appKey: string,
  hotkeyId: string,
  port: number,
  token: string,
  requireToken: boolean,
  lanIp: string | null | undefined,
) {
  const body = JSON.stringify({ app: appKey, hotkey: hotkeyId }, null, 2);
  const authHeader = requireToken
    ? `  -H "Authorization: Bearer ${token}" \\\n`
    : "";

  return `curl -X POST ${apiBaseUrl(port, lanIp)}/send-hotkey \\
${authHeader}  -H "Content-Type: application/json" \\
  -d '${body.replace(/'/g, "'\\''")}'`;
}

function HotkeyApiSection({
  appKey,
  hotkeys,
  port,
  token,
  requireToken,
  allowGet,
  lanIp,
}: {
  appKey: string;
  hotkeys: AppHotkeyBinding[];
  port: number;
  token: string;
  requireToken: boolean;
  allowGet: boolean;
  lanIp?: string | null;
}) {
  const displayToken = requireToken ? token : "";

  return (
    <div className="space-y-4 border-t pt-4">
      <div>
        <p className="font-medium">Phím tắt — /send-hotkey</p>
        <p className="mt-1 text-xs text-muted-foreground">
          Gọi API để focus app và gửi phím đã cấu hình. Trên macOS cần quyền Accessibility
          (Dashboard → Settings).
        </p>
      </div>

      {hotkeys.map((binding) => {
        const getUrl = apiGetUrl(
          port,
          "/send-hotkey",
          { app: appKey, hotkey: binding.id },
          requireToken,
          displayToken,
          lanIp,
        );
        const curlPost = buildSendHotkeyCurl(
          appKey,
          binding.id,
          port,
          token,
          requireToken,
          lanIp,
        );
        const bodyJson = JSON.stringify({ app: appKey, hotkey: binding.id }, null, 2);
        const actionLabel = binding.action === "open" ? "Mở app" : "Gửi phím";

        return (
          <div key={binding.id} className="space-y-3 rounded-lg border border-border/60 p-3">
            <div>
              <p className="font-medium">{binding.name}</p>
              <p className="text-xs text-muted-foreground">
                ID: <span className="font-mono">{binding.id}</span>
                {" · "}
                {actionLabel}
              </p>
            </div>

            {allowGet && <CopyBlock label="GET — link URL" text={getUrl} />}
            <CopyBlock label="POST — curl" text={curlPost} />
            <CopyBlock label="POST — body (JSON)" text={bodyJson} />
          </div>
        );
      })}
    </div>
  );
}

export function AppApiGuide({
  appKey,
  entry,
  port,
  token,
  requireToken,
  allowGet,
  lanIp,
}: AppApiGuideProps) {
  const sampleUrl = entry.url ?? "https://example.com";
  const showUrlExample = appSupportsUrl(entry.path, entry.url_enabled) || Boolean(entry.url);
  const displayToken = requireToken ? token : "";

  const getUrl = apiGetUrl(
    port,
    "/open-app",
    { app: appKey, ...(entry.url ? { url: entry.url } : {}) },
    requireToken,
    displayToken,
    lanIp,
  );
  const getUrlOverride = apiGetUrl(
    port,
    "/open-app",
    { app: appKey, url: sampleUrl },
    requireToken,
    displayToken,
    lanIp,
  );

  const bodyJson = JSON.stringify(
    { app: appKey, ...(entry.url ? { url: entry.url } : {}) },
    null,
    2,
  );
  const curlPost = buildOpenAppCurl(appKey, port, token, requireToken, lanIp, entry.url);
  const curlPostOverride = buildOpenAppCurl(
    appKey,
    port,
    token,
    requireToken,
    lanIp,
    sampleUrl,
  );
  const hotkeys = entry.hotkeys ?? [];

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button
          size="sm"
          className="bg-green-600 text-white hover:bg-green-700"
        >
          API
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[85vh] max-w-2xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>API — {appKey}</DialogTitle>
          <DialogDescription>
            Gọi HTTP API từ thiết bị trong mạng LAN
            {lanIp ? ` (IP: ${lanIp})` : ""}.
            {!requireToken && " Token đang tắt — không cần header Authorization."}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 text-sm">
          {allowGet && (
            <>
              <CopyBlock label="GET — link URL (đơn giản nhất)" text={getUrl} />
              {showUrlExample && (
                <CopyBlock label="GET — ghi đè URL" text={getUrlOverride} />
              )}
            </>
          )}

          <CopyBlock label="POST — curl" text={curlPost} />

          {showUrlExample && (
            <CopyBlock label="POST — curl (ghi đè URL)" text={curlPostOverride} />
          )}

          <CopyBlock label="POST — body (JSON)" text={bodyJson} />

          {hotkeys.length > 0 && (
            <HotkeyApiSection
              appKey={appKey}
              hotkeys={hotkeys}
              port={port}
              token={token}
              requireToken={requireToken}
              allowGet={allowGet}
              lanIp={lanIp}
            />
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
