export interface HotkeyFaIcon {
  id: string;
  label: string;
}

/** Kho icon Font Awesome phổ biến cho phím tắt app. */
export const HOTKEY_FA_ICONS: HotkeyFaIcon[] = [
  { id: "floppy-disk", label: "Save" },
  { id: "copy", label: "Copy" },
  { id: "paste", label: "Paste" },
  { id: "scissors", label: "Cut" },
  { id: "rotate-left", label: "Undo" },
  { id: "rotate-right", label: "Redo" },
  { id: "file-export", label: "Export" },
  { id: "file-import", label: "Import" },
  { id: "download", label: "Download" },
  { id: "upload", label: "Upload" },
  { id: "print", label: "Print" },
  { id: "magnifying-glass", label: "Search" },
  { id: "folder", label: "Folder" },
  { id: "folder-open", label: "Open folder" },
  { id: "file", label: "File" },
  { id: "image", label: "Image" },
  { id: "film", label: "Video" },
  { id: "music", label: "Music" },
  { id: "play", label: "Play" },
  { id: "pause", label: "Pause" },
  { id: "stop", label: "Stop" },
  { id: "plus", label: "Add" },
  { id: "minus", label: "Remove" },
  { id: "trash", label: "Delete" },
  { id: "pen", label: "Edit" },
  { id: "share-nodes", label: "Share" },
  { id: "link", label: "Link" },
  { id: "lock", label: "Lock" },
  { id: "lock-open", label: "Unlock" },
  { id: "gear", label: "Settings" },
  { id: "house", label: "Home" },
  { id: "star", label: "Star" },
  { id: "heart", label: "Favorite" },
  { id: "bolt", label: "Action" },
  { id: "keyboard", label: "Keyboard" },
  { id: "desktop", label: "Desktop" },
  { id: "expand", label: "Maximize" },
  { id: "compress", label: "Minimize" },
  { id: "arrow-right", label: "Next" },
  { id: "arrow-left", label: "Back" },
  { id: "check", label: "Confirm" },
  { id: "xmark", label: "Close" },
  { id: "camera", label: "Camera" },
  { id: "envelope", label: "Mail" },
  { id: "cloud", label: "Cloud" },
  { id: "chart-line", label: "Chart" },
  { id: "code", label: "Code" },
  { id: "terminal", label: "Terminal" },
  { id: "bookmark", label: "Bookmark" },
  { id: "tag", label: "Tag" },
  { id: "filter", label: "Filter" },
  { id: "sort", label: "Sort" },
  { id: "layer-group", label: "Layers" },
  { id: "palette", label: "Color" },
  { id: "crop", label: "Crop" },
  { id: "wand-magic-sparkles", label: "Magic" },
  { id: "eraser", label: "Eraser" },
];

export function faIconRef(id: string): string {
  return `fa:${id}`;
}

export function parseHotkeyIcon(icon?: string | null): {
  type: "fa" | "custom" | null;
  value: string;
} {
  if (!icon) {
    return { type: null, value: "" };
  }
  if (icon.startsWith("fa:")) {
    return { type: "fa", value: icon.slice(3) };
  }
  if (icon.startsWith("custom:")) {
    return { type: "custom", value: icon.slice(7) };
  }
  return { type: null, value: "" };
}

export function faClassName(iconId: string): string {
  return `fa-solid fa-${iconId}`;
}
