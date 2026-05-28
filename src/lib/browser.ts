const BROWSER_HINTS = [
  "chrome",
  "chromium",
  "firefox",
  "safari",
  "edge",
  "brave",
  "opera",
  "arc",
  "vivaldi",
  "zen",
  "browser",
  "tor",
  "waterfox",
  "librewolf",
];

export function isBrowserApp(path: string): boolean {
  const lower = path.toLowerCase();
  return BROWSER_HINTS.some((hint) => lower.includes(hint));
}

export function shouldShowAppUrl(path: string, urlEnabled?: boolean): boolean {
  return Boolean(urlEnabled) || isBrowserApp(path);
}

export function appSupportsUrl(path: string, urlEnabled?: boolean): boolean {
  return shouldShowAppUrl(path, urlEnabled);
}

export function supportsBrowserProfiles(path: string): boolean {
  const lower = path.toLowerCase();
  if (!lower.trim() || lower.includes("safari")) {
    return false;
  }
  return isBrowserApp(path);
}

export function validateAppUrl(url: string): string | null {
  const trimmed = url.trim();
  if (!trimmed) {
    return null;
  }
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    return null;
  }
  return "URL phải bắt đầu bằng http:// hoặc https://";
}
