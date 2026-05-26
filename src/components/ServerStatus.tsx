import { useCallback, useEffect, useState } from "react";
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
  type StatusResponse,
} from "@/lib/tauri";

export function ServerStatus() {
  const [port, setPort] = useState(8765);
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [online, setOnline] = useState(false);
  const [uptime, setUptime] = useState(0);
  const [restarting, setRestarting] = useState(false);
  const [error, setError] = useState<string | null>(null);
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
    </div>
  );
}
