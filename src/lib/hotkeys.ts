const SPECIAL_KEYS: Record<string, string> = {
  " ": "Space",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Delete: "Delete",
  Backspace: "Backspace",
  Enter: "Enter",
  Escape: "Escape",
  Tab: "Tab",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
  Insert: "Insert",
  CapsLock: "CapsLock",
  NumLock: "NumLock",
  ScrollLock: "ScrollLock",
  Pause: "Pause",
  PrintScreen: "PrintScreen",
};

const CODE_TO_KEY: Record<string, string> = {
  Space: "Space",
  Delete: "Delete",
  Backspace: "Backspace",
  Enter: "Enter",
  Escape: "Escape",
  Tab: "Tab",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
  Insert: "Insert",
};

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);

const REGISTERABLE_SPECIAL = new Set([
  "Space",
  "Delete",
  "Backspace",
  "Enter",
  "Escape",
  "Tab",
  "Up",
  "Down",
  "Left",
  "Right",
  "Home",
  "End",
  "PageUp",
  "PageDown",
  "Insert",
  "F1",
  "F2",
  "F3",
  "F4",
  "F5",
  "F6",
  "F7",
  "F8",
  "F9",
  "F10",
  "F11",
  "F12",
]);

function normalizeKeyFromCode(code: string): string | null {
  if (code.startsWith("Key") && code.length === 4) {
    return code.slice(3);
  }
  if (code.startsWith("Digit") && code.length === 6) {
    return code.slice(5);
  }
  if (/^F\d{1,2}$/.test(code)) {
    return code;
  }
  return CODE_TO_KEY[code] ?? null;
}

function normalizeKeyFromLegacyKey(key: string): string | null {
  if (SPECIAL_KEYS[key]) {
    return SPECIAL_KEYS[key];
  }
  if (/^F\d{1,2}$/.test(key)) {
    return key;
  }
  if (key.length === 1 && /^[A-Za-z0-9]$/.test(key)) {
    return key.toUpperCase();
  }
  return null;
}

export function normalizeAcceleratorKeyPart(keyPart: string): string | null {
  const trimmed = keyPart.trim();
  if (REGISTERABLE_SPECIAL.has(trimmed)) {
    return trimmed;
  }
  if (trimmed.length === 1 && /^[A-Za-z0-9]$/.test(trimmed)) {
    return trimmed.toUpperCase();
  }
  return null;
}

export function isValidAccelerator(accelerator: string): boolean {
  const parts = accelerator.split("+").map((part) => part.trim()).filter(Boolean);
  if (parts.length < 2) {
    return false;
  }

  const keyPart = parts[parts.length - 1];
  const modifiers = parts.slice(0, -1);
  if (modifiers.length === 0) {
    return false;
  }

  const validModifiers = new Set([
    "CommandOrControl",
    "CmdOrCtrl",
    "Command",
    "Cmd",
    "Control",
    "Ctrl",
    "Shift",
    "Alt",
    "Option",
  ]);

  if (!modifiers.every((part) => validModifiers.has(part))) {
    return false;
  }

  return normalizeAcceleratorKeyPart(keyPart) !== null;
}

export function keyEventToAccelerator(event: KeyboardEvent): string | null {
  if (MODIFIER_KEYS.has(event.key)) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) {
    parts.push("CommandOrControl");
  }
  if (event.shiftKey) {
    parts.push("Shift");
  }
  if (event.altKey) {
    parts.push("Alt");
  }

  if (parts.length === 0) {
    return null;
  }

  const key = normalizeKeyFromCode(event.code) ?? normalizeKeyFromLegacyKey(event.key);
  if (!key || !normalizeAcceleratorKeyPart(key)) {
    return null;
  }

  parts.push(key);
  return parts.join("+");
}

export function formatAcceleratorDisplay(accelerator: string): string {
  const isMac = navigator.userAgent.toLowerCase().includes("mac");
  return accelerator
    .split("+")
    .map((part) => {
      switch (part) {
        case "CommandOrControl":
          return isMac ? "⌘" : "Ctrl";
        case "Shift":
          return isMac ? "⇧" : "Shift";
        case "Alt":
          return isMac ? "⌥" : "Alt";
        case "Control":
          return isMac ? "⌃" : "Ctrl";
        default:
          return part;
      }
    })
    .join(isMac ? "" : " + ");
}
