const SUPPORTED_CODE_TOKENS: ReadonlySet<string> = new Set([
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "AudioVolumeDown",
  "AudioVolumeMute",
  "AudioVolumeUp",
  "Backquote",
  "Backslash",
  "Backspace",
  "BracketLeft",
  "BracketRight",
  "CapsLock",
  "Comma",
  "Delete",
  "End",
  "Enter",
  "Equal",
  "Escape",
  "Home",
  "Insert",
  "MediaPause",
  "MediaPlay",
  "MediaPlayPause",
  "MediaStop",
  "MediaTrackNext",
  "MediaTrackPrevious",
  "Minus",
  "NumLock",
  "NumpadAdd",
  "NumpadDecimal",
  "NumpadDivide",
  "NumpadEnter",
  "NumpadEqual",
  "NumpadMultiply",
  "NumpadSubtract",
  "PageDown",
  "PageUp",
  "Pause",
  "Period",
  "PrintScreen",
  "Quote",
  "ScrollLock",
  "Semicolon",
  "Slash",
  "Space",
  "Tab",
]);

const TOKEN_LABELS: Record<string, string> = {
  alt: "Alt",
  arrowdown: "Down",
  arrowleft: "Left",
  arrowright: "Right",
  arrowup: "Up",
  audiovolumedown: "Vol-",
  audiovolumemute: "Mute",
  audiovolumeup: "Vol+",
  backquote: "`",
  backslash: "\\",
  backspace: "Backspace",
  bracketleft: "[",
  bracketright: "]",
  capslock: "CapsLock",
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
  mediapause: "Pause",
  mediaplay: "Play",
  mediaplaypause: "Play/Pause",
  mediastop: "Stop",
  mediatracknext: "Next",
  mediatrackprevious: "Prev",
  meta: "Super",
  minus: "-",
  numadd: "NumAdd",
  numdecimal: "NumDecimal",
  numdivide: "NumDivide",
  numenter: "NumEnter",
  numequal: "NumEqual",
  numlock: "NumLock",
  nummultiply: "NumMultiply",
  numpadadd: "NumAdd",
  numpaddecimal: "NumDecimal",
  numpaddivide: "NumDivide",
  numpadenter: "NumEnter",
  numpadequal: "NumEqual",
  numpadmultiply: "NumMultiply",
  numpadsubtract: "NumSubtract",
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

const MODIFIER_CODES: ReadonlySet<string> = new Set([
  "ControlLeft", "ControlRight",
  "AltLeft", "AltRight",
  "ShiftLeft", "ShiftRight",
  "MetaLeft", "MetaRight",
]);

export function isModifierCode(code: string) {
  return MODIFIER_CODES.has(code);
}

export function isModifierKey(key: string) {
  return key === "Alt" || key === "Control" || key === "Meta" || key === "Shift";
}

/**
 * Build an accelerator string by merging two sources of modifier state:
 *
 * 1. `heldCodes` - physical key codes tracked via keydown/keyup on window.
 *    Fixes Chromium/WebView2 misreporting event.ctrlKey on non-US layouts
 *    (e.g. AZERTY Ctrl+Shift+digit loses Ctrl).
 *
 * 2. `event` - the native KeyboardEvent modifier flags.
 *    Fixes Windows sending a synthetic Shift keyup right before numpad keys when
 *    NumLock is on, which causes heldCodes to lose Shift prematurely.
 *
 * A modifier is included if EITHER source reports it as held.
 */
export function buildAcceleratorFromHeldCodes(heldCodes: ReadonlySet<string>, event: KeyboardEvent): string | null {
  const key = keyTokenFromCode(event.code);
  if (!key) {
    return null;
  }

  const modifiers = [
    (event.ctrlKey || heldCodes.has("ControlLeft") || heldCodes.has("ControlRight")) ? "Ctrl" : null,
    (event.altKey || heldCodes.has("AltLeft") || heldCodes.has("AltRight")) ? "Alt" : null,
    (event.shiftKey || heldCodes.has("ShiftLeft") || heldCodes.has("ShiftRight")) ? "Shift" : null,
    (event.metaKey || heldCodes.has("MetaLeft") || heldCodes.has("MetaRight")) ? "Super" : null,
  ].filter((value): value is string => value !== null);

  return [...modifiers, key].join("+");
}

export function formatShortcutForDisplay(accelerator: string | null | undefined) {
  return formatShortcutTokensForDisplay(accelerator).join("+");
}

const MODIFIER_ORDER: Record<string, number> = { Ctrl: 0, Alt: 1, Shift: 2, Super: 3 };

export function formatShortcutTokensForDisplay(accelerator: string | null | undefined) {
  if (!accelerator) {
    return [];
  }

  const tokens = accelerator
    .split("+")
    .map((token) => formatToken(token))
    .filter((token): token is string => Boolean(token));

  const modifiers = tokens.filter((token) => token in MODIFIER_ORDER);
  const keys = tokens.filter((token) => !(token in MODIFIER_ORDER));
  modifiers.sort((a, b) => MODIFIER_ORDER[a] - MODIFIER_ORDER[b]);
  return [...modifiers, ...keys];
}

function keyTokenFromCode(code: string): string | null {
  if (/^Key[A-Z]$/i.test(code)) {
    return `Key${code.slice(3).toUpperCase()}`;
  }

  if (/^Digit\d$/i.test(code)) {
    return `Digit${code.slice(5)}`;
  }

  if (/^Numpad\d$/i.test(code)) {
    return `Numpad${code.slice(6)}`;
  }

  if (/^F\d{1,2}$/i.test(code)) {
    return code.toUpperCase();
  }

  return SUPPORTED_CODE_TOKENS.has(code) ? code : null;
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
