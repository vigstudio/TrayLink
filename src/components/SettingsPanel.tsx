import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
import {
  getAutostartEnabled,
  getConfig,
  regenerateToken,
  restartServer,
  setAutostart,
  updateConfig,
} from "@/lib/tauri";

function authHeader(requireToken: boolean, token: string, showToken: boolean) {
  if (!requireToken) {
    return "";
  }
  return `  -H "Authorization: Bearer ${showToken ? token : "<token>"}" \\\n`;
}

export function SettingsPanel() {
  const [port, setPort] = useState("8765");
  const [token, setToken] = useState("");
  const [requireToken, setRequireToken] = useState(false);
  const [showToken, setShowToken] = useState(false);
  const [autostart, setAutostartState] = useState(false);
  const [message, setMessage] = useState("");

  const load = async () => {
    const config = await getConfig();
    setPort(String(config.port));
    setToken(config.token);
    setRequireToken(config.require_token ?? false);
    const enabled = await getAutostartEnabled();
    setAutostartState(enabled);
  };

  useEffect(() => {
    load();
  }, []);

  const applyPort = async () => {
    const config = await getConfig();
    const nextPort = Number(port);
    if (!nextPort || nextPort < 1024 || nextPort > 65535) {
      setMessage("Port không hợp lệ (1024-65535)");
      return;
    }
    await updateConfig({ ...config, port: nextPort });
    await restartServer();
    setMessage("Đã cập nhật port và restart server");
  };

  const handleRequireToken = async (enabled: boolean) => {
    const config = await getConfig();
    let nextToken = config.token;
    if (enabled && !nextToken) {
      nextToken = await regenerateToken();
      setToken(nextToken);
    }
    await updateConfig({ ...config, require_token: enabled, token: nextToken });
    setRequireToken(enabled);
    setMessage(enabled ? "Đã bật xác thực token" : "Đã tắt token — API mở cho LAN");
  };

  const handleRegenerateToken = async () => {
    const newToken = await regenerateToken();
    setToken(newToken);
    setMessage("Đã tạo token mới");
  };

  const copyToken = async () => {
    await navigator.clipboard.writeText(token);
    setMessage("Đã copy token");
  };

  const handleAutostart = async (enabled: boolean) => {
    await setAutostart(enabled);
    setAutostartState(enabled);
    setMessage(enabled ? "Đã bật autostart" : "Đã tắt autostart");
  };

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Cài đặt</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <Label htmlFor="port">Port</Label>
            <div className="flex gap-2">
              <Input id="port" value={port} onChange={(e) => setPort(e.target.value)} />
              <Button onClick={applyPort}>Apply</Button>
            </div>
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="require-token">Yêu cầu token API</Label>
              <p className="text-sm text-muted-foreground">
                Tắt (mặc định) để dùng trên mạng LAN không cần Authorization header
              </p>
            </div>
            <Switch
              id="require-token"
              checked={requireToken}
              onCheckedChange={handleRequireToken}
            />
          </div>

          {requireToken && (
            <div className="space-y-2">
              <Label>API Token</Label>
              <div className="flex flex-wrap gap-2">
                <Input
                  readOnly
                  className="min-w-[200px] flex-1"
                  value={showToken ? token : "•".repeat(Math.min(token.length, 24))}
                />
                <Button variant="outline" onClick={() => setShowToken((v) => !v)}>
                  {showToken ? "Ẩn" : "Hiện"}
                </Button>
                <Button variant="outline" onClick={copyToken}>
                  Copy
                </Button>
                <Button variant="destructive" onClick={handleRegenerateToken}>
                  Regenerate
                </Button>
              </div>
            </div>
          )}

          <Separator />

          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="autostart">Autostart khi boot</Label>
              <p className="text-sm text-muted-foreground">
                Tự khởi động TrayLink cùng hệ thống
              </p>
            </div>
            <Switch id="autostart" checked={autostart} onCheckedChange={handleAutostart} />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Ví dụ API (curl)</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 font-mono text-xs">
          <pre className="overflow-x-auto rounded-md bg-muted p-3">{`curl http://127.0.0.1:${port}/status`}</pre>
          <pre className="overflow-x-auto rounded-md bg-muted p-3">{`curl -X POST http://127.0.0.1:${port}/open-app \\
${authHeader(requireToken, token, showToken)}  -H "Content-Type: application/json" \\
  -d '{"app":"my-app"}'`}</pre>
          <pre className="overflow-x-auto rounded-md bg-muted p-3">{`curl -X POST http://127.0.0.1:${port}/open-file \\
${authHeader(requireToken, token, showToken)}  -H "Content-Type: application/json" \\
  -d '{"path":"/path/to/file.mp4"}'`}</pre>
          <pre className="overflow-x-auto rounded-md bg-muted p-3">{`curl -X POST http://127.0.0.1:${port}/exec \\
${authHeader(requireToken, token, showToken)}  -H "Content-Type: application/json" \\
  -d '{"cmd":"restart_server"}'`}</pre>
        </CardContent>
      </Card>

      {message && <p className="text-sm text-muted-foreground">{message}</p>}
    </div>
  );
}
