import type { AppConfig, RemoteDeckLayout } from "@/lib/tauri";

export type DeckItemType = "app" | "cmd";

export interface DeckEditorItem {
  key: string;
  label: string;
  path?: string;
  visible: boolean;
  type: DeckItemType;
  customIcon?: string | null;
}

export function slotId(type: DeckItemType, key: string): string {
  return `${type}:${key}`;
}

export function parseSlotId(slot: string): { type: DeckItemType; key: string } | null {
  const separator = slot.indexOf(":");
  if (separator <= 0) return null;

  const type = slot.slice(0, separator);
  const key = slot.slice(separator + 1);
  if ((type === "app" || type === "cmd") && key) {
    return { type, key };
  }
  return null;
}

export function appDisplayLabel(key: string, path?: string): string {
  if (path) {
    const match = path.match(/([^/\\]+?)(?:\.app|\.exe)?$/i);
    if (match?.[1] && match[1] !== key) {
      return match[1];
    }
  }
  return key.replace(/_/g, " ");
}

export function applyOrder(allKeys: string[], order: string[]): string[] {
  const keySet = new Set(allKeys);
  const result: string[] = [];

  for (const key of order) {
    if (keySet.has(key)) {
      result.push(key);
    }
  }

  const remaining = allKeys
    .filter((key) => !result.includes(key))
    .sort((a, b) => a.localeCompare(b, undefined, { sensitivity: "base" }));

  return [...result, ...remaining];
}

function buildDisplayOrderFromLegacy(layout: RemoteDeckLayout): string[] {
  const appKeys = applyOrder([], layout.app_order);
  const cmdKeys = applyOrder([], layout.command_order);
  return [
    ...appKeys.map((key) => slotId("app", key)),
    ...cmdKeys.map((key) => slotId("cmd", key)),
  ];
}

function resolveDisplayOrder(config: AppConfig): string[] {
  const layout = config.remote_deck ?? emptyRemoteDeckLayout();
  if (layout.display_order.length > 0) {
    return layout.display_order;
  }

  const appKeys = applyOrder(Object.keys(config.apps), layout.app_order);
  const cmdKeys = applyOrder(Object.keys(config.commands), layout.command_order);
  return [
    ...appKeys.map((key) => slotId("app", key)),
    ...cmdKeys.map((key) => slotId("cmd", key)),
  ];
}

export function buildDeckItems(config: AppConfig): DeckEditorItem[] {
  const layout = config.remote_deck ?? emptyRemoteDeckLayout();
  const hiddenApps = new Set(layout.hidden_apps);
  const hiddenCommands = new Set(layout.hidden_commands);
  const displayOrder = resolveDisplayOrder(config);
  const seen = new Set<string>();

  const items: DeckEditorItem[] = [];

  for (const slot of displayOrder) {
    const parsed = parseSlotId(slot);
    if (!parsed || seen.has(slot)) continue;

    if (parsed.type === "app" && config.apps[parsed.key]) {
      seen.add(slot);
      items.push({
        key: parsed.key,
        label: appDisplayLabel(parsed.key, config.apps[parsed.key]?.path),
        path: config.apps[parsed.key]?.path,
        visible: !hiddenApps.has(parsed.key),
        type: "app",
        customIcon: layout.custom_icons?.[slot] ?? null,
      });
      continue;
    }

    if (parsed.type === "cmd" && config.commands[parsed.key]) {
      seen.add(slot);
      items.push({
        key: parsed.key,
        label: parsed.key.replace(/_/g, " "),
        visible: !hiddenCommands.has(parsed.key),
        type: "cmd",
        customIcon: layout.custom_icons?.[slot] ?? null,
      });
    }
  }

  for (const key of Object.keys(config.apps)) {
    const slot = slotId("app", key);
    if (seen.has(slot)) continue;
    seen.add(slot);
    items.push({
      key,
      label: appDisplayLabel(key, config.apps[key]?.path),
      path: config.apps[key]?.path,
      visible: !hiddenApps.has(key),
      type: "app",
      customIcon: layout.custom_icons?.[slot] ?? null,
    });
  }

  for (const key of Object.keys(config.commands)) {
    const slot = slotId("cmd", key);
    if (seen.has(slot)) continue;
    seen.add(slot);
    items.push({
      key,
      label: key.replace(/_/g, " "),
      visible: !hiddenCommands.has(key),
      type: "cmd",
      customIcon: layout.custom_icons?.[slot] ?? null,
    });
  }

  return items;
}

export function buildAppDeckItems(config: AppConfig): DeckEditorItem[] {
  return buildDeckItems(config).filter((item) => item.type === "app");
}

export function buildCommandDeckItems(config: AppConfig): DeckEditorItem[] {
  return buildDeckItems(config).filter((item) => item.type === "cmd");
}

export function layoutFromItems(
  items: DeckEditorItem[],
  existing?: RemoteDeckLayout,
): RemoteDeckLayout {
  const layout = existing ?? emptyRemoteDeckLayout();

  return {
    display_order: items.map((item) => slotId(item.type, item.key)),
    app_order: items.filter((item) => item.type === "app").map((item) => item.key),
    command_order: items.filter((item) => item.type === "cmd").map((item) => item.key),
    hidden_apps: items.filter((item) => item.type === "app" && !item.visible).map((item) => item.key),
    hidden_commands: items
      .filter((item) => item.type === "cmd" && !item.visible)
      .map((item) => item.key),
    custom_icons: { ...layout.custom_icons },
  };
}

export function emptyRemoteDeckLayout(): RemoteDeckLayout {
  return {
    display_order: [],
    app_order: [],
    command_order: [],
    hidden_apps: [],
    hidden_commands: [],
    custom_icons: {},
  };
}

export function syncRemoteDeckOnAppAdd(
  layout: RemoteDeckLayout,
  key: string,
): RemoteDeckLayout {
  const slot = slotId("app", key);
  if (layout.display_order.includes(slot)) {
    return layout;
  }
  return {
    ...layout,
    display_order: [...layout.display_order, slot],
    app_order: layout.app_order.includes(key)
      ? layout.app_order
      : [...layout.app_order, key],
  };
}

export function syncRemoteDeckOnAppRemove(
  layout: RemoteDeckLayout,
  key: string,
): RemoteDeckLayout {
  const slot = slotId("app", key);
  const { [slot]: _removed, ...customIcons } = layout.custom_icons ?? {};
  return {
    display_order: layout.display_order.filter((item) => item !== slot),
    app_order: layout.app_order.filter((item) => item !== key),
    command_order: layout.command_order,
    hidden_apps: layout.hidden_apps.filter((item) => item !== key),
    hidden_commands: layout.hidden_commands,
    custom_icons: customIcons,
  };
}

export function syncRemoteDeckOnCommandAdd(
  layout: RemoteDeckLayout,
  key: string,
): RemoteDeckLayout {
  const slot = slotId("cmd", key);
  if (layout.display_order.includes(slot)) {
    return layout;
  }
  return {
    ...layout,
    display_order: [...layout.display_order, slot],
    command_order: layout.command_order.includes(key)
      ? layout.command_order
      : [...layout.command_order, key],
  };
}

export function syncRemoteDeckOnCommandRemove(
  layout: RemoteDeckLayout,
  key: string,
): RemoteDeckLayout {
  const slot = slotId("cmd", key);
  const { [slot]: _removed, ...customIcons } = layout.custom_icons ?? {};
  return {
    display_order: layout.display_order.filter((item) => item !== slot),
    app_order: layout.app_order,
    command_order: layout.command_order.filter((item) => item !== key),
    hidden_apps: layout.hidden_apps,
    hidden_commands: layout.hidden_commands.filter((item) => item !== key),
    custom_icons: customIcons,
  };
}

export function moveItem<T>(items: T[], from: number, to: number): T[] {
  if (from === to || from < 0 || to < 0 || from >= items.length || to >= items.length) {
    return items;
  }
  const next = [...items];
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item);
  return next;
}

export function moveItemBySlot(
  items: DeckEditorItem[],
  fromSlot: string,
  toSlot: string,
): DeckEditorItem[] {
  const from = items.findIndex((item) => slotId(item.type, item.key) === fromSlot);
  const to = items.findIndex((item) => slotId(item.type, item.key) === toSlot);
  if (from < 0 || to < 0) {
    return items;
  }
  return moveItem(items, from, to);
}

export function reorderVisibleSlots(
  items: DeckEditorItem[],
  fromSlot: string,
  toSlot: string,
): DeckEditorItem[] {
  if (fromSlot === toSlot) {
    return items;
  }

  const hiddenItems = items.filter((item) => !item.visible);
  if (hiddenItems.length === 0) {
    return moveItemBySlot(items, fromSlot, toSlot);
  }

  const visibleSlots = items.filter((item) => item.visible).map((item) => slotId(item.type, item.key));
  const fromVisible = visibleSlots.indexOf(fromSlot);
  const toVisible = visibleSlots.indexOf(toSlot);
  if (fromVisible < 0 || toVisible < 0) {
    return moveItemBySlot(items, fromSlot, toSlot);
  }

  const nextVisibleSlots = moveItem(visibleSlots, fromVisible, toVisible);
  const itemBySlot = new Map(items.map((item) => [slotId(item.type, item.key), item]));

  return [
    ...nextVisibleSlots
      .map((slot) => itemBySlot.get(slot))
      .filter((item): item is DeckEditorItem => Boolean(item)),
    ...hiddenItems,
  ];
}

export function reorderTypeItems(
  allItems: DeckEditorItem[],
  type: DeckItemType,
  orderedKeys: string[],
): DeckEditorItem[] {
  const typeIndices: number[] = [];
  allItems.forEach((item, index) => {
    if (item.type === type) {
      typeIndices.push(index);
    }
  });

  const typedItems = orderedKeys
    .map((key) => allItems.find((item) => item.type === type && item.key === key))
    .filter((item): item is DeckEditorItem => Boolean(item));

  const next = [...allItems];
  typeIndices.forEach((index, i) => {
    if (typedItems[i]) {
      next[index] = typedItems[i];
    }
  });
  return next;
}

export function migrateRemoteDeckLayout(layout: RemoteDeckLayout): RemoteDeckLayout {
  if (layout.display_order.length > 0) {
    return layout;
  }
  return {
    ...layout,
    display_order: buildDisplayOrderFromLegacy(layout),
  };
}
