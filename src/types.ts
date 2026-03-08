export type HiddenAppSummary = {
  appName: string;
  windowCount: number;
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
};
