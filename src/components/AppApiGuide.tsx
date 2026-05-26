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
import { apiGetUrl, type AppEntry } from "@/lib/tauri";

interface AppApiGuideProps {
  appKey: string;
  entry: AppEntry;
  port: number;
  token: string;
  requireToken: boolean;
  allowGet: boolean;
}

function buildOpenAppCurl(
  appKey: string,
  port: number,
  token: string,
  requireToken: boolean,
  url?: string,
) {
  const body = url
    ? JSON.stringify({ app: appKey, url }, null, 2)
    : JSON.stringify({ app: appKey }, null, 2);

  const authHeader = requireToken
    ? `  -H "Authorization: Bearer ${token}" \\\n`
    : "";

  return `curl -X POST http://127.0.0.1:${port}/open-app \\
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
}: AppApiGuideProps) {
  const maskedToken = token ? `${token.slice(0, 8)}...` : "—";
  const sampleUrl = entry.url ?? "https://example.com";
  const showUrlExample = isBrowserApp(entry.path) || Boolean(entry.url);
  const displayToken = requireToken ? token : "";

  const getUrl = apiGetUrl(
    port,
    "/open-app",
    { app: appKey, ...(entry.url ? { url: entry.url } : {}) },
    requireToken,
    displayToken,
  );
  const getUrlOverride = apiGetUrl(
    port,
    "/open-app",
    { app: appKey, url: sampleUrl },
    requireToken,
    displayToken,
  );

  const headersBlock = requireToken
    ? `Authorization: Bearer ${maskedToken}\nContent-Type: application/json`
    : "Content-Type: application/json";

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
            Gọi HTTP API từ script, Stream Deck, hoặc thiết bị trên mạng LAN.
            {!requireToken && " Token đang tắt — không cần header Authorization."}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 text-sm">
          {allowGet && (
            <div>
              <p className="mb-1 font-medium">GET — link URL (đơn giản nhất)</p>
              <code className="block break-all rounded-md bg-muted p-2 text-xs">
                {getUrl}
              </code>
              {showUrlExample && (
                <>
                  <p className="mb-1 mt-3 font-medium">GET — ghi đè URL</p>
                  <code className="block break-all rounded-md bg-muted p-2 text-xs">
                    {getUrlOverride}
                  </code>
                </>
              )}
            </div>
          )}

          <div>
            <p className="mb-1 font-medium">POST — Endpoint</p>
            <code className="block rounded-md bg-muted p-2 text-xs">
              POST http://127.0.0.1:{port}/open-app
            </code>
          </div>

          <div>
            <p className="mb-1 font-medium">Headers</p>
            <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs">{headersBlock}</pre>
          </div>

          <div>
            <p className="mb-1 font-medium">Body</p>
            <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs">
              {JSON.stringify({ app: appKey, ...(entry.url ? { url: entry.url } : {}) }, null, 2)}
            </pre>
          </div>

          <div>
            <p className="mb-1 font-medium">curl (POST)</p>
            <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs">
              {buildOpenAppCurl(appKey, port, token, requireToken, entry.url)}
            </pre>
          </div>

          {showUrlExample && (
            <div>
              <p className="mb-1 font-medium">curl (POST, ghi đè URL)</p>
              <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs">
                {buildOpenAppCurl(appKey, port, token, requireToken, sampleUrl)}
              </pre>
            </div>
          )}

          <div>
            <p className="mb-1 font-medium">Response thành công</p>
            <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs">
              {JSON.stringify({ ok: true, message: `opened app '${appKey}'` }, null, 2)}
            </pre>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
