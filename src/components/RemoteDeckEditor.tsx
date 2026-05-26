import { useCallback, useEffect, useRef, useState } from "react";
import { Smartphone } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { RemoteDeckQrDialog } from "@/components/RemoteDeckQrDialog";
import { DeckPreviewSortable, DeckSectionSortable } from "@/components/deck-dnd";
import {
  browseDeckIconPath,
  clearDeckIcon,
  getConfig,
  getServerStatus,
  remoteDeckHttpsUrl,
  setDeckIconFromFile,
  updateConfig,
  type AppConfig,
} from "@/lib/tauri";
import {
  buildDeckItems,
  layoutFromItems,
  moveItem,
  reorderTypeItems,
  type DeckEditorItem,
} from "@/lib/remote-deck";
import { invalidateDeckIconCache } from "@/lib/icon-cache";

function DeckSection({
  title,
  description,
  type,
  sectionItems,
  allItems,
  onSetItems,
  onPersist,
  onPickIcon,
  onResetIcon,
}: {
  title: string;
  description: string;
  type: "app" | "cmd";
  sectionItems: DeckEditorItem[];
  allItems: DeckEditorItem[];
  onSetItems: (items: DeckEditorItem[]) => void;
  onPersist: () => void;
  onPickIcon: (item: DeckEditorItem) => void;
  onResetIcon: (item: DeckEditorItem) => void;
}) {
  if (sectionItems.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{title}</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">{description}</p>
          <p className="mt-2 text-sm text-muted-foreground">
            Chưa có mục nào — thêm trong tab Apps & Commands.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <p className="text-sm text-muted-foreground">{description}</p>
      </CardHeader>
      <CardContent>
        <DeckSectionSortable
          sectionItems={sectionItems}
          allItems={allItems}
          type={type}
          onSetItems={onSetItems}
          onPersist={onPersist}
          onPickIcon={onPickIcon}
          onResetIcon={onResetIcon}
          reorderTypeItems={reorderTypeItems}
          moveItem={moveItem}
        />
      </CardContent>
    </Card>
  );
}

export function RemoteDeckEditor({ active = true }: { active?: boolean }) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [deckItems, setDeckItems] = useState<DeckEditorItem[]>([]);
  const [remoteUrl, setRemoteUrl] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [dirty, setDirty] = useState(false);
  const deckItemsRef = useRef<DeckEditorItem[]>([]);
  const configRef = useRef<AppConfig | null>(null);
  const dirtyRef = useRef(false);
  const hasLoadedRef = useRef(false);

  const syncDeckItems = useCallback((items: DeckEditorItem[]) => {
    deckItemsRef.current = items;
    setDeckItems(items);
  }, []);

  useEffect(() => {
    configRef.current = config;
  }, [config]);

  useEffect(() => {
    dirtyRef.current = dirty;
  }, [dirty]);

  const load = useCallback(async (options?: { silent?: boolean; force?: boolean }) => {
    const silent = Boolean(options?.silent && hasLoadedRef.current);
    const force = Boolean(options?.force);

    const [data, status] = await Promise.all([getConfig(), getServerStatus()]);
    setConfig(data);
    if (force || !silent || !dirtyRef.current) {
      syncDeckItems(buildDeckItems(data));
    }
    setRemoteUrl(
      remoteDeckHttpsUrl(
        status.https_port ?? data.port + 1,
        Boolean(data.require_token),
        data.require_token ? data.token : "",
        status.lan_ip,
      ),
    );
    if (force || !silent) {
      setDirty(false);
    }
    hasLoadedRef.current = true;
  }, [syncDeckItems]);

  useEffect(() => {
    if (!active) return;
    void load({ silent: hasLoadedRef.current });
  }, [active, load]);

  const appItems = deckItems.filter((item) => item.type === "app");
  const commandItems = deckItems.filter((item) => item.type === "cmd");

  const persistLayout = useCallback(async (items: DeckEditorItem[], successMessage?: string) => {
    const currentConfig = configRef.current;
    if (!currentConfig) return;

    setSaving(true);
    setMessage("");
    try {
      const next: AppConfig = {
        ...currentConfig,
        remote_deck: layoutFromItems(items, currentConfig.remote_deck),
      };
      await updateConfig(next);
      configRef.current = next;
      syncDeckItems(items);
      setConfig(next);
      setDirty(false);
      setMessage(successMessage ?? "Đã lưu bố cục Remote Deck");
    } catch (err) {
      setMessage(String(err));
    } finally {
      setSaving(false);
    }
  }, [syncDeckItems]);

  const markDirty = () => setDirty(true);

  const save = async () => {
    await persistLayout(deckItemsRef.current);
  };

  const mergeConfigWithLocalLayout = useCallback(
    (remoteConfig: AppConfig, localItems: DeckEditorItem[]): AppConfig => ({
      ...remoteConfig,
      remote_deck: {
        ...layoutFromItems(localItems, remoteConfig.remote_deck),
        custom_icons: remoteConfig.remote_deck?.custom_icons ?? {},
      },
    }),
    [],
  );

  const handlePickIcon = async (item: DeckEditorItem) => {
    try {
      const sourcePath = await browseDeckIconPath();
      if (!sourcePath) return;

      const localItems = deckItemsRef.current;
      if (dirty) {
        await persistLayout(localItems, "Đang lưu bố cục...");
      }

      await setDeckIconFromFile(item.type, item.key, sourcePath);
      invalidateDeckIconCache(item.type, item.key);
      const refreshed = await getConfig();
      const merged = mergeConfigWithLocalLayout(refreshed, localItems);
      if (JSON.stringify(merged.remote_deck) !== JSON.stringify(refreshed.remote_deck)) {
        await updateConfig(merged);
      }
      setConfig(merged);
      syncDeckItems(buildDeckItems(merged));
      setMessage(`Đã đổi icon cho ${item.label}`);
    } catch (err) {
      setMessage(String(err));
    }
  };

  const handleResetIcon = async (item: DeckEditorItem) => {
    try {
      const localItems = deckItemsRef.current;
      if (dirty) {
        await persistLayout(localItems, "Đang lưu bố cục...");
      }

      await clearDeckIcon(item.type, item.key);
      invalidateDeckIconCache(item.type, item.key);
      const refreshed = await getConfig();
      const merged = mergeConfigWithLocalLayout(refreshed, localItems);
      if (JSON.stringify(merged.remote_deck) !== JSON.stringify(refreshed.remote_deck)) {
        await updateConfig(merged);
      }
      setConfig(merged);
      syncDeckItems(buildDeckItems(merged));
      setMessage(`Đã khôi phục icon mặc định cho ${item.label}`);
    } catch (err) {
      setMessage(String(err));
    }
  };

  const handlePreviewReorder = useCallback(
    (nextItems: DeckEditorItem[]) => {
      deckItemsRef.current = nextItems;
      setDeckItems(nextItems);
      markDirty();
      void persistLayout(nextItems);
    },
    [persistLayout],
  );

  const applyLocalChange = useCallback((items: DeckEditorItem[]) => {
    deckItemsRef.current = items;
    setDeckItems(items);
    markDirty();
  }, []);

  const handlePersist = useCallback(() => {
    void persistLayout(deckItemsRef.current);
  }, [persistLayout]);

  if (!config) {
    return <p className="text-sm text-muted-foreground">Đang tải...</p>;
  }

  const hasItems = deckItems.length > 0;

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Smartphone className="size-5" />
            Xem trước Remote Deck
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-muted-foreground">
            Kéo nút ≡ để sắp xếp — tự động lưu khi thả. Click icon để đổi hình. Dùng ↑/↓ nếu cần.
          </p>
          {hasItems ? (
            <DeckPreviewSortable
              items={deckItems}
              onReorder={handlePreviewReorder}
              onPickIcon={handlePickIcon}
            />
          ) : (
            <p className="text-sm text-muted-foreground">
              Thêm app hoặc command trong tab Apps & Commands trước.
            </p>
          )}
          {remoteUrl && (
            <div className="flex flex-wrap items-center gap-2">
              <p className="text-xs text-muted-foreground">
                Link Remote Deck:{" "}
                <code className="rounded bg-muted px-1 py-0.5">{remoteUrl}</code>
              </p>
              <RemoteDeckQrDialog url={remoteUrl} buttonLabel="Quét QR" />
            </div>
          )}
        </CardContent>
      </Card>

      <DeckSection
        title="Apps"
        description="Thứ tự và hiển thị các app trên Remote Deck."
        type="app"
        sectionItems={appItems}
        allItems={deckItems}
        onSetItems={applyLocalChange}
        onPersist={handlePersist}
        onPickIcon={handlePickIcon}
        onResetIcon={handleResetIcon}
      />

      <DeckSection
        title="Commands"
        description="Thứ tự và hiển thị các lệnh trên Remote Deck."
        type="cmd"
        sectionItems={commandItems}
        allItems={deckItems}
        onSetItems={applyLocalChange}
        onPersist={handlePersist}
        onPickIcon={handlePickIcon}
        onResetIcon={handleResetIcon}
      />

      <div className="flex flex-wrap items-center gap-3">
        <Button onClick={save} disabled={saving || !dirty || !hasItems}>
          {saving ? "Đang lưu..." : "Lưu bố cục"}
        </Button>
        <Button variant="outline" onClick={() => void load({ force: true })} disabled={saving}>
          Hoàn tác thay đổi
        </Button>
        {message && <p className="text-sm text-muted-foreground">{message}</p>}
      </div>
    </div>
  );
}
