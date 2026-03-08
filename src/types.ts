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

export type Display = {
  id: string;
  name: string;
  width: number;
  height: number;
  x: number;
  y: number;
  scaleFactor: number;
  isPrimary: boolean;
  isBlackedOut: boolean;
  hostsPanel: boolean;
  canBlackout: boolean;
  hiddenApps: HiddenAppSummary[];
};

export type DisplayUpdatePayload = {
  displays: Display[];
  activeDisplayCount: number;
  blackoutCount: number;
  allowCursorExitActiveDisplays: boolean;
  showOverlayHiddenApps: boolean;
};
