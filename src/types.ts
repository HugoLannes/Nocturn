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
};

export type DisplayUpdatePayload = {
  displays: Display[];
  activeDisplayCount: number;
  blackoutCount: number;
  allowCursorExitActiveDisplays: boolean;
};
