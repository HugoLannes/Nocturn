/**
 * Browser-only dev harness.
 *
 * Loaded by main.tsx ONLY when the app runs outside the Tauri runtime
 * (plain `npm run dev` opened in a browser). Mocks the IPC surface with an
 * interactive in-memory display state so the UI can be exercised without
 * the Rust backend. Never ships in the Tauri build.
 */
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { Display, DisplayUpdatePayload, ShortcutSettings } from "../types";

mockWindows("main");

type MockDisplay = Display;

const displays: MockDisplay[] = [
  {
    id: "mock-1",
    name: "\\\\.\\DISPLAY1",
    manufacturer: "Dell",
    model: "U2723QE",
    persistentKey: "edid:1234:5678:mock-1",
    width: 2560,
    height: 1440,
    x: 0,
    y: 0,
    scaleFactor: 1,
    refreshRate: 60,
    orientation: 0,
    isPrimary: true,
    isBlackedOut: false,
    hostsPanel: true,
    canBlackout: true,
    hotkey: "ctrl+shift+numpad1",
    hiddenApps: [],
  },
  {
    id: "mock-2",
    name: "\\\\.\\DISPLAY2",
    manufacturer: "LG",
    model: "27GP950",
    persistentKey: "edid:9abc:def0:mock-2",
    width: 3840,
    height: 2160,
    x: 2560,
    y: -360,
    scaleFactor: 1,
    refreshRate: 144,
    orientation: 0,
    isPrimary: false,
    isBlackedOut: true,
    hostsPanel: false,
    canBlackout: true,
    hotkey: "ctrl+shift+numpad2",
    hiddenApps: [
      { appName: "Discord", windowCount: 1 },
      { appName: "Spotify", windowCount: 2 },
    ],
  },
  {
    id: "mock-3",
    name: "\\\\.\\DISPLAY3",
    manufacturer: "AOC",
    model: "24G2U",
    persistentKey: "edid:5555:aaaa:mock-3",
    width: 1920,
    height: 1080,
    x: -1920,
    y: 200,
    scaleFactor: 1,
    refreshRate: 144,
    orientation: 0,
    isPrimary: false,
    isBlackedOut: false,
    hostsPanel: false,
    canBlackout: true,
    hotkey: null,
    hiddenApps: [],
  },
];

let allowCursorExitActiveDisplays = false;
let showOverlayHiddenApps = true;
let focusModeHotkey: string | null = "ctrl+shift+numpad0";

function shortcutSettings(): ShortcutSettings {
  return {
    focusModeHotkey,
    displayBindings: displays
      .filter((d) => d.hotkey)
      .map((d) => ({
        displayKey: d.persistentKey,
        displayLabel: `${d.manufacturer} ${d.model}`,
        accelerator: d.hotkey as string,
        isAvailable: true,
      })),
  };
}

function payload(): DisplayUpdatePayload {
  const active = displays.filter((d) => !d.isBlackedOut).length;
  return {
    displays: structuredClone(displays),
    activeDisplayCount: active,
    blackoutCount: displays.length - active,
    allowCursorExitActiveDisplays,
    showOverlayHiddenApps,
    shortcutSettings: shortcutSettings(),
  };
}

mockIPC((cmd, args) => {
  switch (cmd) {
    case "get_displays":
      return payload();
    case "toggle_display": {
      const id = (args as { id?: string })?.id;
      const target = displays.find((d) => d.id === id);
      if (!target) throw `Unknown display ${id}`;
      const activeCount = displays.filter((d) => !d.isBlackedOut).length;
      if (!target.isBlackedOut && activeCount <= 1) {
        throw "At least one display must stay active.";
      }
      target.isBlackedOut = !target.isBlackedOut;
      return target.id;
    }
    case "unblank_all":
      displays.forEach((d) => (d.isBlackedOut = false));
      return null;
    case "focus_primary": {
      const anyBlackedOut = displays.some((d) => d.isBlackedOut);
      displays.forEach((d) => (d.isBlackedOut = anyBlackedOut ? false : !d.isPrimary));
      return null;
    }
    case "set_allow_cursor_exit_active_displays":
      allowCursorExitActiveDisplays = Boolean((args as { allowed?: boolean })?.allowed);
      return payload();
    case "set_show_overlay_hidden_apps":
      showOverlayHiddenApps = Boolean((args as { enabled?: boolean })?.enabled);
      return payload();
    case "reset_to_defaults":
      allowCursorExitActiveDisplays = false;
      showOverlayHiddenApps = true;
      focusModeHotkey = "ctrl+shift+numpad0";
      return payload();
    case "set_shortcut_settings": {
      const input = (args as { hotkeys?: { focusModeHotkey: string | null } })?.hotkeys;
      if (input) focusModeHotkey = input.focusModeHotkey;
      return payload();
    }
    case "get_overlay_card_presentation":
      return null;
    case "hide_window":
    case "close_app":
      console.info(`[dev-mocks] ${cmd} called`);
      return null;
    default:
      // Plugin surface (event listen/unlisten, updater check, …): succeed silently.
      if (cmd.startsWith("plugin:event|")) return 0;
      if (cmd.startsWith("plugin:updater|")) return null;
      console.debug(`[dev-mocks] unhandled command: ${cmd}`, args);
      return null;
  }
});

console.info("[dev-mocks] Tauri IPC mocked — browser dev mode with 3 fake displays");
