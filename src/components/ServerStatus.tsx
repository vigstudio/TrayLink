import { useCallback, useEffect, useState } from "react";
import { Check, Copy, Smartphone } from "lucide-react";
import { isTauri } from "@tauri-apps/api/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  formatUptime,
  getConfig,
  getServerStatus,
  getServerUptime,
  restartServer,
  apiBaseUrl,
  remoteDeckUrl,
  remoteDeckHttpsUrl,
  type StatusResponse,
} from "@/lib/tauri";
import { RemoteDeckQrDialog } from "@/components/RemoteDeckQrDialog";

export function ServerStatus() {
  const [port, setPort] = useState(8765);
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [online, setOnline] = useState(false);
  const [uptime, setUptime] = useState(0);
  const [restarting, setRestarting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [remoteUrl, setRemoteUrl] = useState<string | null>(null);
  const [remoteHttpsUrl, setRemoteHttpsUrl] = useState<string | null>(null);
  const [copiedRemote, setCopiedRemote] = useState(false);
  const [copiedRemoteHttps, setCopiedRemoteHttps] = useState(false);
  const inTauri = isTauri();

  const refresh = useCallback(async () => {
    try {
      setError(null);

      if (inTauri) {
        const config = await getConfig();
        setPort(config.port);
        const data = await getServerStatus();
        setStatus(data);
        setOnline(data.online);
        setError(data.error ?? null);
        const requireToken = Boolean(config.require_token);
        const token = config.require_token ? config.token : "";
        setRemoteUrl(remoteDeckUrl(config.port, requireToken, token, data.lan_ip));
        setRemoteHttpsUrl(
          remoteDeckHttpsUrl(data.https_port ?? config.port + 1, requireToken, token, data.lan_ip),
        );
        const seconds = await getServerUptime();
        setUptime(seconds);
        return;
      }

      const data = await getServerStatus(port);
      setPort(data.port);
      setStatus(data);
      setOnline(data.online);
    } catch (err) {
      setOnline(false);
      setStatus(null);
      setError(String(err));
      setUptime(0);
    }
  }, [inTauri, port]);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 3000);
    return () => clearInterval(interval);
  }, [refresh]);

  const handleCopyRemote = async () => {
    if (!remoteUrl) return;
    await navigator.clipboard.writeText(remoteUrl);
    setCopiedRemote(true);
    window.setTimeout(() => setCopiedRemote(false), 2000);
  };

  const handleCopyRemoteHttps = async () => {
    if (!remoteHttpsUrl) return;
    await navigator.clipboard.writeText(remoteHttpsUrl);
    setCopiedRemoteHttps(true);
    window.setTimeout(() => setCopiedRemoteHttps(false), 2000);
  };

  const handleRestart = async () => {
    if (!inTauri) {
      setError("Restart chỉ dùng được trong app TrayLink (không phải trình duyệt).");
      return;
    }

    setRestarting(true);
    try {
      await restartServer();
      await refresh();
    } finally {
      setRestarting(false);
    }
  };

  return (
    <div className="grid gap-4 md:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            Trạng thái server
            <Badge variant={online ? "default" : "destructive"}>
              {online ? "Online" : "Offline"}
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <p>
            <span className="text-muted-foreground">Version:</span>{" "}
            {status?.version ?? "—"}
          </p>
          <p>
            <span className="text-muted-foreground">API port:</span> {port}
          </p>
          <p>
            <span className="text-muted-foreground">Uptime:</span>{" "}
            {inTauri ? formatUptime(uptime) : "— (mở qua TrayLink app)"}
          </p>
          <p>
            <span className="text-muted-foreground">API (LAN):</span>{" "}
            {apiBaseUrl(port, status?.lan_ip)}
          </p>
          <p>
            <span className="text-muted-foreground">API (máy này):</span>{" "}
            http://127.0.0.1:{port}
          </p>
          <p className="text-muted-foreground">
            UI dev chạy trên port <strong>1420</strong> — đó là giao diện, không phải API.
          </p>
          {error && <p className="text-destructive">{error}</p>}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Điều khiển</CardTitle>
        </CardHeader>
        <CardContent>
          <Button onClick={handleRestart} disabled={restarting || !inTauri}>
            {restarting ? "Đang restart..." : "Restart Server"}
          </Button>
          <p className="mt-3 text-sm text-muted-foreground">
            Server lắng nghe trên mạng LAN (0.0.0.0). Thiết bị khác gọi bằng IP LAN ở trên,
            port {port} — không phải port UI dev (1420).
          </p>
        </CardContent>
      </Card>

      {remoteHttpsUrl && (
        <Card className="md:col-span-2 border-primary/30">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Smartphone className="size-5" />
              Remote Deck — HTTPS (giữ màn hình sáng)
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <p className="text-sm text-muted-foreground">
              Dùng link <strong>HTTPS</strong> trên điện thoại để Wake Lock API hoạt động
              (giữ màn hình không tắt). Lần đầu Safari/Chrome sẽ hỏi chấp nhận chứng chỉ — chọn
              Tiếp tục / Advanced → Proceed.
            </p>
            <div className="flex flex-wrap items-center gap-2">
              <code className="flex-1 break-all rounded-md bg-muted px-3 py-2 text-xs">
                {remoteHttpsUrl}
              </code>
              <Button variant="outline" size="sm" onClick={handleCopyRemoteHttps}>
                {copiedRemoteHttps ? (
                  <>
                    <Check className="size-3.5" />
                    Đã copy
                  </>
                ) : (
                  <>
                    <Copy className="size-3.5" />
                    Copy HTTPS
                  </>
                )}
              </Button>
              <RemoteDeckQrDialog url={remoteHttpsUrl} buttonLabel="QR HTTPS" />
            </div>
          </CardContent>
        </Card>
      )}

      {remoteUrl && (
        <Card className="md:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-muted-foreground">
              <Smartphone className="size-5" />
              Remote Deck — HTTP (API thường)
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <p className="text-sm text-muted-foreground">
              HTTP không hỗ trợ Wake Lock — màn hình có thể vẫn tự tắt. Ưu tiên link HTTPS ở
              trên cho điện thoại.
            </p>
            <div className="flex flex-wrap items-center gap-2">
              <code className="flex-1 break-all rounded-md bg-muted px-3 py-2 text-xs">
                {remoteUrl}
              </code>
              <Button variant="outline" size="sm" onClick={handleCopyRemote}>
                {copiedRemote ? (
                  <>
                    <Check className="size-3.5" />
                    Đã copy
                  </>
                ) : (
                  <>
                    <Copy className="size-3.5" />
                    Copy HTTP
                  </>
                )}
              </Button>
              <RemoteDeckQrDialog url={remoteUrl} buttonLabel="QR HTTP" />
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
