import { getAppIcon, getDeckIconDataUrl } from "@/lib/tauri";

type DeckIconKind = "app" | "cmd";

const appIconCache = new Map<string, string | null>();
const appIconPending = new Map<string, Promise<string | null>>();

const deckIconCache = new Map<string, string | null>();
const deckIconPending = new Map<string, Promise<string | null>>();

function deckIconKey(type: DeckIconKind, key: string): string {
  return `${type}:${key}`;
}

export function peekAppIconCache(path: string): string | null | undefined {
  if (!appIconCache.has(path)) return undefined;
  return appIconCache.get(path) ?? null;
}

export async function getAppIconCached(path: string): Promise<string | null> {
  if (appIconCache.has(path)) {
    return appIconCache.get(path) ?? null;
  }

  let pending = appIconPending.get(path);
  if (!pending) {
    pending = getAppIcon(path)
      .then((url) => {
        appIconCache.set(path, url ?? null);
        appIconPending.delete(path);
        return url ?? null;
      })
      .catch(() => {
        appIconCache.set(path, null);
        appIconPending.delete(path);
        return null;
      });
    appIconPending.set(path, pending);
  }

  return pending;
}

export function invalidateAppIconCache(path?: string) {
  if (path) {
    appIconCache.delete(path);
    appIconPending.delete(path);
    return;
  }
  appIconCache.clear();
  appIconPending.clear();
}

export function peekDeckIconCache(type: DeckIconKind, key: string): string | null | undefined {
  const cacheKey = deckIconKey(type, key);
  if (!deckIconCache.has(cacheKey)) return undefined;
  return deckIconCache.get(cacheKey) ?? null;
}

export async function getDeckIconCached(type: DeckIconKind, key: string): Promise<string | null> {
  const cacheKey = deckIconKey(type, key);
  if (deckIconCache.has(cacheKey)) {
    return deckIconCache.get(cacheKey) ?? null;
  }

  let pending = deckIconPending.get(cacheKey);
  if (!pending) {
    pending = getDeckIconDataUrl(type, key)
      .then((url) => {
        deckIconCache.set(cacheKey, url ?? null);
        deckIconPending.delete(cacheKey);
        return url ?? null;
      })
      .catch(() => {
        deckIconCache.set(cacheKey, null);
        deckIconPending.delete(cacheKey);
        return null;
      });
    deckIconPending.set(cacheKey, pending);
  }

  return pending;
}

export function invalidateDeckIconCache(type?: DeckIconKind, key?: string) {
  if (type && key) {
    const cacheKey = deckIconKey(type, key);
    deckIconCache.delete(cacheKey);
    deckIconPending.delete(cacheKey);
    return;
  }
  deckIconCache.clear();
  deckIconPending.clear();
}
