const CODE_LABELS: Record<string, string> = {
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  ArrowUp: "Up",
  Backquote: "`",
  Backslash: "\\",
  BracketLeft: "[",
  BracketRight: "]",
  Comma: ",",
  Delete: "Delete",
  End: "End",
  Enter: "Enter",
  Equal: "=",
  Escape: "Esc",
  Home: "Home",
  Insert: "Insert",
  Minus: "-",
  NumpadAdd: "NumAdd",
  NumpadDecimal: "NumDecimal",
  NumpadDivide: "NumDivide",
  NumpadEnter: "NumEnter",
  NumpadEqual: "NumEqual",
  NumpadMultiply: "NumMultiply",
  NumpadSubtract: "NumSubtract",
  PageDown: "PageDown",
  PageUp: "PageUp",
  Pause: "Pause",
  Period: ".",
  PrintScreen: "PrintScreen",
  Quote: "'",
  ScrollLock: "ScrollLock",
  Semicolon: ";",
  Slash: "/",
  Space: "Space",
  Tab: "Tab",
};

const TOKEN_LABELS: Record<string, string> = {
  alt: "Alt",
  arrowdown: "Down",
  arrowleft: "Left",
  arrowright: "Right",
  arrowup: "Up",
  backquote: "`",
  backslash: "\\",
  bracketleft: "[",
  bracketright: "]",
  cmd: "Super",
  cmdorcontrol: "Ctrl",
  cmdorctrl: "Ctrl",
  comma: ",",
  command: "Super",
  commandorcontrol: "Ctrl",
  commandorctrl: "Ctrl",
  control: "Ctrl",
  ctrl: "Ctrl",
  delete: "Delete",
  down: "Down",
  end: "End",
  enter: "Enter",
  equal: "=",
  escape: "Esc",
  esc: "Esc",
  home: "Home",
  insert: "Insert",
  left: "Left",
  meta: "Super",
  minus: "-",
  numadd: "NumAdd",
  numdecimal: "NumDecimal",
  numdivide: "NumDivide",
  numenter: "NumEnter",
  numequal: "NumEqual",
  nummultiply: "NumMultiply",
  numsubtract: "NumSubtract",
  option: "Alt",
  pagedown: "PageDown",
  pageup: "PageUp",
  pause: "Pause",
  period: ".",
  printscreen: "PrintScreen",
  quote: "'",
  right: "Right",
  scrolllock: "ScrollLock",
  semicolon: ";",
  shift: "Shift",
  slash: "/",
  space: "Space",
  super: "Super",
  tab: "Tab",
  up: "Up",
};

export function isModifierKey(key: string) {
  return key === "Alt" || key === "Control" || key === "Meta" || key === "Shift";
}

export function buildAcceleratorFromKeyboardEvent(event: KeyboardEvent): string | null {
  const key = keyLabelFromCode(event.code);
  if (!key) {
    return null;
  }

  const modifiers = [
    event.ctrlKey ? "Ctrl" : null,
    event.altKey ? "Alt" : null,
    event.shiftKey ? "Shift" : null,
    event.metaKey ? "Super" : null,
  ].filter((value): value is string => value !== null);

  if (modifiers.length === 0) {
    return null;
  }

  return [...modifiers, key].join("+");
}

export function formatShortcutForDisplay(accelerator: string | null | undefined) {
  return formatShortcutTokensForDisplay(accelerator).join("+");
}

export function formatShortcutTokensForDisplay(accelerator: string | null | undefined) {
  if (!accelerator) {
    return [];
  }

  return accelerator
    .split("+")
    .map((token) => formatToken(token))
    .filter((token): token is string => Boolean(token));
}

function keyLabelFromCode(code: string): string | null {
  if (/^Key[A-Z]$/i.test(code)) {
    return code.slice(3).toUpperCase();
  }

  if (/^Digit\d$/i.test(code)) {
    return code.slice(5);
  }

  if (/^Numpad\d$/i.test(code)) {
    return `Num${code.slice(6)}`;
  }

  if (/^F\d{1,2}$/i.test(code)) {
    return code.toUpperCase();
  }

  return CODE_LABELS[code] ?? null;
}

function formatToken(rawToken: string) {
  const token = rawToken.trim();
  if (!token) {
    return "";
  }

  const normalized = token.toLowerCase();
  if (TOKEN_LABELS[normalized]) {
    return TOKEN_LABELS[normalized];
  }

  if (/^key[a-z]$/i.test(token)) {
    return token.slice(3).toUpperCase();
  }

  if (/^digit\d$/i.test(token)) {
    return token.slice(5);
  }

  if (/^numpad\d$/i.test(token)) {
    return `Num${token.slice(6)}`;
  }

  if (/^f\d{1,2}$/i.test(token)) {
    return token.toUpperCase();
  }

  return token.length === 1 ? token.toUpperCase() : token;
}
