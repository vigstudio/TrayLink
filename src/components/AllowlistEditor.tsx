import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { getConfig, testOpenApp, updateConfig, type AppConfig } from "@/lib/tauri";
import { AppPathPicker } from "@/components/AppPathPicker";
import { AppApiGuide } from "@/components/AppApiGuide";
import { AppIcon } from "@/components/AppIcon";
import { isBrowserApp, validateAppUrl } from "@/lib/browser";

export function AllowlistEditor() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [appKey, setAppKey] = useState("");
  const [appPath, setAppPath] = useState("");
  const [appUrl, setAppUrl] = useState("");
  const [cmdKey, setCmdKey] = useState("");
  const [cmdWin, setCmdWin] = useState("");
  const [cmdMac, setCmdMac] = useState("");
  const [cmdLinux, setCmdLinux] = useState("");
  const [saving, setSaving] = useState(false);
  const [testingApp, setTestingApp] = useState<string | null>(null);
  const [message, setMessage] = useState("");

  const showUrlField = isBrowserApp(appPath);

  const load = async () => {
    const data = await getConfig();
    setConfig(data);
  };

  useEffect(() => {
    load();
  }, []);

  useEffect(() => {
    if (!showUrlField) {
      setAppUrl("");
    }
  }, [showUrlField]);

  const save = async (next: AppConfig) => {
    setSaving(true);
    setMessage("");
    try {
      await updateConfig(next);
      setConfig(next);
      setMessage("Đã lưu allowlist");
    } catch (err) {
      setMessage(String(err));
    } finally {
      setSaving(false);
    }
  };

  const addApp = async () => {
    if (!config || !appKey || !appPath) return;

    const urlError = validateAppUrl(appUrl);
    if (urlError) {
      setMessage(urlError);
      return;
    }

    const next = {
      ...config,
      apps: {
        ...config.apps,
        [appKey]: {
          path: appPath,
          args: [],
          url: appUrl.trim() || undefined,
        },
      },
    };
    await save(next);
    setAppKey("");
    setAppPath("");
    setAppUrl("");
  };

  const removeApp = async (key: string) => {
    if (!config) return;
    const apps = { ...config.apps };
    delete apps[key];
    await save({ ...config, apps });
  };

  const handleTestApp = async (key: string) => {
    setTestingApp(key);
    setMessage("");
    try {
      const result = await testOpenApp(key);
      setMessage(result);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setTestingApp(null);
    }
  };

  const addCommand = async () => {
    if (!config || !cmdKey) return;
    const next = {
      ...config,
      commands: {
        ...config.commands,
        [cmdKey]: {
          win: cmdWin || undefined,
          mac: cmdMac || undefined,
          linux: cmdLinux || undefined,
        },
      },
    };
    await save(next);
    setCmdKey("");
    setCmdWin("");
    setCmdMac("");
    setCmdLinux("");
  };

  const removeCommand = async (key: string) => {
    if (!config) return;
    const commands = { ...config.commands };
    delete commands[key];
    await save({ ...config, commands });
  };

  if (!config) {
    return <p className="text-sm text-muted-foreground">Đang tải...</p>;
  }

  const appEntries = Object.entries(config.apps);

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Apps đã đăng ký</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-12"></TableHead>
                <TableHead>Key</TableHead>
                <TableHead>Path</TableHead>
                <TableHead>URL</TableHead>
                <TableHead className="text-right">Thao tác</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {appEntries.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-center text-muted-foreground">
                    Chưa có app nào — thêm app bên dưới
                  </TableCell>
                </TableRow>
              ) : (
                appEntries.map(([key, entry]) => (
                  <TableRow key={key}>
                    <TableCell>
                      <AppIcon path={entry.path} name={key} />
                    </TableCell>
                    <TableCell className="font-medium">{key}</TableCell>
                    <TableCell className="max-w-[240px] truncate font-mono text-xs">
                      {entry.path}
                    </TableCell>
                    <TableCell className="max-w-[180px] truncate text-xs text-muted-foreground">
                      {entry.url ?? "—"}
                    </TableCell>
                    <TableCell>
                      <div className="flex justify-end gap-2">
                        <Button
                          size="sm"
                          className="bg-blue-600 text-white hover:bg-blue-700"
                          disabled={testingApp === key}
                          onClick={() => handleTestApp(key)}
                        >
                          {testingApp === key ? "..." : "Test"}
                        </Button>
                        <AppApiGuide
                          appKey={key}
                          entry={entry}
                          port={config.port}
                          token={config.token}
                          requireToken={config.require_token ?? false}
                          allowGet={config.allow_get ?? true}
                        />
                        <Button variant="destructive" size="sm" onClick={() => removeApp(key)}>
                          Xóa
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>

          <div className="grid gap-3 md:grid-cols-3">
            <div className="space-y-2">
              <Label htmlFor="app-key">App key</Label>
              <Input id="app-key" value={appKey} onChange={(e) => setAppKey(e.target.value)} />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="app-path">Path / App name</Label>
              <AppPathPicker
                id="app-path"
                value={appPath}
                onChange={setAppPath}
                onNamePick={(name) => {
                  if (!appKey) {
                    setAppKey(name);
                  }
                }}
              />
            </div>
          </div>

          {showUrlField && (
            <div className="space-y-2">
              <Label htmlFor="app-url">URL mặc định (tùy chọn)</Label>
              <Input
                id="app-url"
                value={appUrl}
                onChange={(e) => setAppUrl(e.target.value)}
                placeholder="https://example.com"
              />
              <p className="text-xs text-muted-foreground">
                Trình duyệt sẽ mở URL này khi gọi API. Có thể ghi đè bằng{" "}
                <code className="rounded bg-muted px-1">{`{"app":"...","url":"..."}`}</code>
              </p>
            </div>
          )}

          <Button onClick={addApp} disabled={saving}>
            Thêm app
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Command whitelist</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Key</TableHead>
                <TableHead>Windows</TableHead>
                <TableHead>macOS</TableHead>
                <TableHead>Linux</TableHead>
                <TableHead></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {Object.entries(config.commands).map(([key, entry]) => (
                <TableRow key={key}>
                  <TableCell>{key}</TableCell>
                  <TableCell className="text-xs">{entry.win ?? (entry.internal ? "internal" : "—")}</TableCell>
                  <TableCell className="text-xs">{entry.mac ?? "—"}</TableCell>
                  <TableCell className="text-xs">{entry.linux ?? "—"}</TableCell>
                  <TableCell>
                    <Button variant="destructive" size="sm" onClick={() => removeCommand(key)}>
                      Xóa
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>

          <div className="grid gap-3 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="cmd-key">Command key</Label>
              <Input id="cmd-key" value={cmdKey} onChange={(e) => setCmdKey(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="cmd-win">Windows</Label>
              <Input id="cmd-win" value={cmdWin} onChange={(e) => setCmdWin(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="cmd-mac">macOS</Label>
              <Input id="cmd-mac" value={cmdMac} onChange={(e) => setCmdMac(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="cmd-linux">Linux</Label>
              <Input id="cmd-linux" value={cmdLinux} onChange={(e) => setCmdLinux(e.target.value)} />
            </div>
          </div>
          <Button onClick={addCommand} disabled={saving}>
            Thêm command
          </Button>
        </CardContent>
      </Card>

      {message && <p className="text-sm text-muted-foreground">{message}</p>}
    </div>
  );
}
