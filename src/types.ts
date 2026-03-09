export type HiddenAppSummary = {
  appName: string;
  windowCount: number;
};

export type OverlayDock = "top" | "right" | "bottom" | "left" | "center";

export type OverlayCardPresentation = {
  hiddenApps: HiddenAppSummary[];
  dock: OverlayDock;
  isEnabled: boolean;
};

export type DisplayShortcutBinding = {
  displayKey: string;
  displayLabel: string;
  accelerator: string;
};

export type DisplayShortcutBindingInfo = DisplayShortcutBinding & {
  isAvailable: boolean;
};

export type ShortcutSettings = {
  focusModeHotkey: string | null;
  displayBindings: DisplayShortcutBindingInfo[];
};

export type ShortcutSettingsInput = {
  focusModeHotkey: string | null;
  displayBindings: DisplayShortcutBinding[];
};

export type Display = {
  id: string;
  name: string;
  manufacturer: string;
  model: string;
  persistentKey: string;
  width: number;
  height: number;
  x: number;
  y: number;
  scaleFactor: number;
  refreshRate: number;
  orientation: number;
  isPrimary: boolean;
  isBlackedOut: boolean;
  hostsPanel: boolean;
  canBlackout: boolean;
  hotkey: string | null;
  hiddenApps: HiddenAppSummary[];
};

export type DisplayUpdatePayload = {
  displays: Display[];
  activeDisplayCount: number;
  blackoutCount: number;
  allowCursorExitActiveDisplays: boolean;
  showOverlayHiddenApps: boolean;
  shortcutSettings: ShortcutSettings;
};
