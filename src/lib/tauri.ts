import { invoke as tauriInvoke, isTauri } from "@tauri-apps/api/core";

export interface AppEntry {
  path: string;
  args?: string[];
  url?: string;
}

export interface ExecEntry {
  internal?: boolean;
  win?: string;
  mac?: string;
  linux?: string;
}

export interface AppConfig {
  port: number;
  token: string;
  require_token?: boolean;
  allow_get?: boolean;
  autostart: boolean;
  apps: Record<string, AppEntry>;
  commands: Record<string, ExecEntry>;
}

export interface LogEntry {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  duration_ms: number;
  client_ip: string;
}

export interface StatusResponse {
  online: boolean;
  version: string;
  port: number;
  lan_ip?: string | null;
  error?: string | null;
}

export function apiHost(lanIp?: string | null): string {
  return lanIp || "127.0.0.1";
}

export function apiBaseUrl(port: number, lanIp?: string | null): string {
  return `http://${apiHost(lanIp)}:${port}`;
}

export function runningInTauri(): boolean {
  return isTauri();
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error(
      "Dashboard cần mở trong app TrayLink (icon tray → Open Dashboard), không phải trình duyệt.",
    );
  }
  return tauriInvoke<T>(cmd, args);
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function updateConfig(config: AppConfig): Promise<void> {
  return invoke("update_config", { config });
}

export async function restartServer(): Promise<void> {
  return invoke("restart_api_server");
}

export async function getRequestLogs(): Promise<LogEntry[]> {
  return invoke<LogEntry[]>("get_request_logs");
}

export async function regenerateToken(): Promise<string> {
  return invoke<string>("regenerate_token");
}

export async function setAutostart(enabled: boolean): Promise<void> {
  return invoke("set_autostart", { enabled });
}

export async function getAutostartEnabled(): Promise<boolean> {
  return invoke<boolean>("get_autostart_enabled");
}

export async function getServerUptime(): Promise<number> {
  return invoke<number>("get_server_uptime");
}

async function fetchStatusFromApi(port: number): Promise<StatusResponse> {
  const response = await fetch(`http://127.0.0.1:${port}/status`);
  if (!response.ok) {
    throw new Error(`API server không phản hồi trên port ${port}`);
  }
  return response.json();
}

export async function getServerStatus(port = 8765): Promise<StatusResponse> {
  if (isTauri()) {
    return invoke<StatusResponse>("get_server_status");
  }
  return fetchStatusFromApi(port);
}

export interface InstalledApp {
  name: string;
  path: string;
}

export async function listInstalledApps(): Promise<InstalledApp[]> {
  return invoke<InstalledApp[]>("list_installed_apps_cmd");
}

export async function browseAppPath(): Promise<string | null> {
  if (!isTauri()) {
    throw new Error("Chọn app chỉ dùng được trong app TrayLink.");
  }

  const { open } = await import("@tauri-apps/plugin-dialog");
  const isMac = navigator.userAgent.toLowerCase().includes("mac");

  if (isMac) {
    const selected = await open({
      title: "Chọn ứng dụng (.app)",
      multiple: false,
      directory: true,
      defaultPath: "/Applications",
    });
    return typeof selected === "string" ? selected : null;
  }

  const selected = await open({
    title: "Chọn ứng dụng",
    multiple: false,
    directory: false,
    filters: [
      {
        name: "Application",
        extensions: ["exe", "lnk", "desktop"],
      },
    ],
  });

  if (selected === null) {
    return null;
  }

  return typeof selected === "string" ? selected : selected[0] ?? null;
}

export function slugifyAppKey(name: string): string {
  return name
    .replace(/\.app$/i, "")
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

export async function getAppIcon(path: string): Promise<string | null> {
  return invoke<string | null>("get_app_icon", { path });
}

export async function testOpenApp(appKey: string): Promise<string> {
  return invoke<string>("test_open_app", { appKey });
}

export function apiGetUrl(
  port: number,
  path: string,
  params: Record<string, string | undefined>,
  requireToken: boolean,
  token: string,
  lanIp?: string | null,
): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value) {
      search.set(key, value);
    }
  }
  if (requireToken && token) {
    search.set("token", token);
  }
  const query = search.toString();
  return `${apiBaseUrl(port, lanIp)}${path}${query ? `?${query}` : ""}`;
}

export function formatUptime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  return `${h}h ${m}m ${s}s`;
}
