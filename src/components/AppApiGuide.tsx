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
import { isBrowserApp } from "@/lib/browser";
import { apiBaseUrl, apiGetUrl, type AppEntry } from "@/lib/tauri";

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
  const showUrlExample = isBrowserApp(entry.path) || Boolean(entry.url);
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
        </div>
      </DialogContent>
    </Dialog>
  );
}
