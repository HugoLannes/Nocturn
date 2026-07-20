import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState, type CSSProperties } from "react";
import appLogo from "./assets/nocturn-mark.svg";
import { DisplayLayout } from "./components/DisplayLayout";
import { SettingsPage } from "./components/SettingsPage";
import { Tooltip } from "./components/Tooltip";
import { useDisplays } from "./hooks/useDisplays";
import { useUpdater } from "./hooks/useUpdater";
import { cn, monoTextStyle } from "./ui";

type AppView = "displays" | "settings";

const DISPLAY_HEADLINES = [
  "Ready to go dark.",
  "Let's lay back.",
  "Give your eyes a break.",
  "Dim the extra glow.",
  "Time to wind down.",
  "Quiet the bright screens.",
  "Less glow, more calm.",
  "Night mode, softly.",
];

const appShellStyle = {
  background: "var(--bg)",
  backgroundImage:
    "radial-gradient(ellipse 70% 45% at 25% -5%, rgba(var(--accent-rgb), 0.12) 0%, transparent 100%), radial-gradient(ellipse 50% 35% at 85% 105%, rgba(52, 211, 153, 0.06) 0%, transparent 100%)",
} satisfies CSSProperties;

const titlebarLogoStyle = {
  filter: "drop-shadow(0 0 8px rgba(var(--accent-rgb), 0.72)) drop-shadow(0 0 18px rgba(var(--accent-rgb), 0.44))",
} satisfies CSSProperties;

const titlebarNameStyle = {
  filter: "drop-shadow(0 0 10px rgba(var(--accent-rgb), 0.16))",
} satisfies CSSProperties;

const splashStyle = {
  background:
    "radial-gradient(circle at 50% 42%, rgba(var(--accent-rgb), 0.22), transparent 34%), radial-gradient(circle at 50% 50%, rgba(var(--accent-rgb), 0.11), transparent 58%), rgba(8, 8, 13, 0.97)",
  backdropFilter: "blur(14px)",
} satisfies CSSProperties;

const splashMarkStyle = {
  background: "linear-gradient(180deg, rgba(255, 255, 255, 0.04), rgba(255, 255, 255, 0.015))",
  boxShadow: "0 24px 70px rgba(var(--accent-rgb), 0.18)",
} satisfies CSSProperties;

const splashMarkGlowStyle = {
  background: "radial-gradient(circle, rgba(var(--accent-rgb), 0.24), transparent 72%)",
  filter: "blur(6px)",
} satisfies CSSProperties;

const splashLogoStyle = {
  filter: "drop-shadow(0 0 18px rgba(var(--accent-rgb), 0.4))",
} satisfies CSSProperties;

const toolbarButtonClass = "grid h-8 w-8 place-items-center rounded-[10px] border border-white/8 bg-white/[0.035] text-[var(--text-secondary)] transition-[border-color,background,color,box-shadow] duration-[140ms] ease-out hover:border-white/15 hover:bg-white/8 hover:text-[var(--text-primary)] disabled:cursor-default max-[560px]:h-7 max-[560px]:w-7";

const updateToolbarButtonClass = "border-[rgba(var(--accent-rgb),0.28)] text-[var(--accent-soft)] shadow-[0_0_0_1px_rgba(var(--accent-rgb),0.14),0_0_18px_rgba(var(--accent-rgb),0.16)] [background:linear-gradient(180deg,rgba(var(--accent-rgb),0.26)_0%,rgba(var(--accent-rgb),0.18)_100%)] hover:border-[rgba(var(--accent-rgb),0.42)] hover:text-[#f6f3ff] hover:shadow-[0_0_0_1px_rgba(var(--accent-rgb),0.18),0_0_22px_rgba(var(--accent-rgb),0.24)] hover:[background:linear-gradient(180deg,rgba(var(--accent-rgb),0.34)_0%,rgba(var(--accent-rgb),0.24)_100%)] motion-reduce:animate-none";

const updateToolbarBusyClass = "border-[rgba(var(--accent-rgb),0.34)] text-[#f6f3ff] shadow-[0_0_0_1px_rgba(var(--accent-rgb),0.16),0_0_18px_rgba(var(--accent-rgb),0.18)] [background:linear-gradient(180deg,rgba(var(--accent-rgb),0.28)_0%,rgba(var(--accent-rgb),0.2)_100%)]";

const windowControlClass = "grid h-[26px] w-[26px] place-items-center rounded-[6px] border-0 bg-transparent text-[var(--text-secondary)] transition-[background,color] duration-[120ms] ease-out hover:bg-white/8 hover:text-[var(--text-primary)]";

function navSegmentClass(isActive: boolean) {
  return cn(
    "relative z-[1] rounded-[6px] px-[10px] py-[1px] text-[12px] font-semibold leading-[1.3] tracking-[-0.01em] transition-colors duration-[160ms] ease-out",
    isActive
      ? "text-[color:var(--text-primary)]"
      : "text-[color:var(--text-secondary)] hover:bg-white/[0.06] hover:text-[color:var(--text-primary)]",
  );
}

function wakeButtonClass(isActive: boolean) {
  return cn(
    "flex h-10 w-full items-center justify-center rounded-[12px] border px-4 text-[var(--text-primary)] transition-[border-color,background,transform,box-shadow,opacity] duration-[140ms] ease-out enabled:hover:-translate-y-px enabled:hover:border-[rgba(var(--accent-rgb),0.34)] enabled:hover:[background:linear-gradient(180deg,rgba(var(--accent-rgb),0.16),rgba(255,255,255,0.04)_54%),rgba(255,255,255,0.04)] enabled:hover:shadow-[inset_0_1px_0_rgba(255,255,255,0.06),0_10px_24px_rgba(var(--accent-rgb),0.18)] disabled:cursor-not-allowed disabled:opacity-50 max-[560px]:h-9 max-[560px]:px-3",
    isActive
      ? "border-[rgba(var(--accent-rgb),0.28)] [background:linear-gradient(180deg,rgba(var(--accent-rgb),0.18),rgba(255,255,255,0.035)_54%),rgba(255,255,255,0.03)] shadow-[inset_0_1px_0_rgba(255,255,255,0.04),0_8px_24px_rgba(var(--accent-rgb),0.14)]"
      : "border-white/10 [background:linear-gradient(180deg,rgba(255,255,255,0.045),rgba(255,255,255,0.02)_54%),rgba(255,255,255,0.028)]",
  );
}

function App() {
  const [appVersion, setAppVersion] = useState("");
  const {
    displays,
    isLoading,
    isMutating,
    pendingDisplayIds,
    blackoutCount,
    loadError,
    mutationError,
    clearMutationError,
    loadDisplays,
    toggleDisplay,
    restoreAllDisplays,
    focusPrimary,
    allowCursorExitActiveDisplays,
    showOverlayHiddenApps,
    shortcutSettings,
    resetToDefaults,
    setAllowCursorExitActiveDisplays,
    setShowOverlayHiddenApps,
    setShortcutSettings,
    lastActiveDisplayId,
  } = useDisplays();
  const {
    isUpdateAvailable,
    availableVersion,
    installState,
    downloadProgress,
    errorMessage: updaterErrorMessage,
    installUpdate,
  } = useUpdater();
  const [activeView, setActiveView] = useState<AppView>("displays");
  const [showSplash, setShowSplash] = useState(true);
  const [isSplashExiting, setIsSplashExiting] = useState(false);
  const [hasMetMinimumSplash, setHasMetMinimumSplash] = useState(false);
  const [displayHeadline] = useState(
    () => DISPLAY_HEADLINES[Math.floor(Math.random() * DISPLAY_HEADLINES.length)],
  );
  const hasHiddenDisplays = blackoutCount > 0;
  const hasPendingCards = pendingDisplayIds.size > 0;
  const isRestoreAllBusy = isMutating || hasPendingCards;
  const isUpdateBusy = installState === "downloading" || installState === "installing" || installState === "relaunching";
  const shouldShowUpdateButton = isUpdateAvailable;

  let updateTooltipTitle = "A new update is available";
  let updateTooltipDescription = availableVersion ? `Click to install v${availableVersion}` : "Click to install";

  if (installState === "downloading") {
    updateTooltipTitle = "Downloading update";
    updateTooltipDescription = downloadProgress !== null ? `${downloadProgress}% downloaded` : "Preparing the installer...";
  } else if (installState === "installing") {
    updateTooltipTitle = "Installing update";
    updateTooltipDescription = "Nocturn is applying the new version.";
  } else if (installState === "relaunching") {
    updateTooltipTitle = "Restarting Nocturn";
    updateTooltipDescription = "The app will relaunch automatically.";
  } else if (installState === "error") {
    updateTooltipTitle = "Update installation failed";
    updateTooltipDescription = updaterErrorMessage ?? "Click to try again.";
  }

  useEffect(() => {
    let isCancelled = false;

    void getVersion()
      .then((version) => {
        if (!isCancelled) {
          setAppVersion(version);
        }
      })
      .catch((error) => {
        console.error("Failed to read app version:", error);
      });

    return () => {
      isCancelled = true;
    };
  }, []);

  useEffect(() => {
    const minimumSplashTimer = window.setTimeout(() => {
      setHasMetMinimumSplash(true);
    }, 900);

    return () => window.clearTimeout(minimumSplashTimer);
  }, []);

  useEffect(() => {
    if (isLoading || !hasMetMinimumSplash) {
      return;
    }

    setIsSplashExiting(true);

    const removeTimer = window.setTimeout(() => {
      setShowSplash(false);
    }, 280);

    return () => {
      window.clearTimeout(removeTimer);
    };
  }, [hasMetMinimumSplash, isLoading]);

  return (
    <div className="relative flex h-screen flex-col overflow-hidden" style={appShellStyle}>
      {showSplash && (
        <div
          className={cn(
            "absolute inset-0 z-20 flex flex-col items-center justify-center gap-[18px] transition-[opacity,transform] duration-[280ms] ease-out",
            isSplashExiting && "pointer-events-none scale-[1.02] opacity-0",
          )}
          style={splashStyle}
          aria-hidden={isSplashExiting}
        >
          <div className="relative grid h-[112px] w-[112px] place-items-center rounded-[28px] border border-white/8 max-[560px]:h-24 max-[560px]:w-24" style={splashMarkStyle}>
            <div className="absolute inset-[14px] rounded-[22px]" style={splashMarkGlowStyle} aria-hidden="true" />
            <img src={appLogo} alt="Nocturn logo" className="relative z-[1] h-[70px] w-[70px] object-contain max-[560px]:h-[60px] max-[560px]:w-[60px]" style={splashLogoStyle} />
          </div>
          <div className="flex flex-col items-center gap-1">
            <span className="text-[22px] font-bold tracking-[-0.04em] max-[560px]:text-[20px]">Nocturn</span>
            <span className="text-[11px] uppercase tracking-[0.08em] text-[rgba(226,232,240,0.72)]" style={monoTextStyle}>
              Preparing your displays...
            </span>
          </div>
        </div>
      )}

      <header
        className="relative flex h-11 shrink-0 select-none items-center justify-between border-b border-[var(--border)] pl-[14px] pr-[10px] [-webkit-app-region:drag] [app-region:drag]"
        data-tauri-drag-region
      >
        <nav
          className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-[9px] border border-white/8 bg-white/[0.03] p-[2px] [-webkit-app-region:no-drag] [app-region:no-drag]"
          aria-label="Main navigation"
        >
          <div className="relative grid grid-cols-2">
            <div
              className={cn(
                "absolute inset-y-0 left-0 w-1/2 rounded-[6px] bg-[rgba(var(--accent-rgb),0.2)] shadow-[inset_0_1px_0_rgba(255,255,255,0.07),0_0_0_1px_rgba(var(--accent-rgb),0.28)] transition-transform duration-[200ms] ease-out motion-reduce:transition-none",
                activeView === "settings" ? "translate-x-full" : "translate-x-0",
              )}
              aria-hidden="true"
            />
            <button
              type="button"
              className={navSegmentClass(activeView === "displays")}
              onClick={() => setActiveView("displays")}
              aria-current={activeView === "displays" ? "page" : undefined}
            >
              Displays
            </button>
            <button
              type="button"
              className={navSegmentClass(activeView === "settings")}
              onClick={() => setActiveView("settings")}
              aria-current={activeView === "settings" ? "page" : undefined}
            >
              Settings
            </button>
          </div>
        </nav>

        <div className="flex items-center gap-[10px]">
          <button
            type="button"
            className="group flex items-center gap-[10px] rounded-[8px] border-0 bg-transparent p-0 [-webkit-app-region:no-drag] [app-region:no-drag]"
            onClick={() => setActiveView("displays")}
            title="Back to displays"
          >
            <img src={appLogo} alt="Nocturn logo" className="h-7 w-7 object-contain transition-[filter] duration-[160ms] ease-out group-hover:[filter:drop-shadow(0_0_10px_rgba(var(--accent-rgb),0.9))_drop-shadow(0_0_22px_rgba(var(--accent-rgb),0.6))]" style={titlebarLogoStyle} />
            <span
              className="inline-flex items-center bg-[linear-gradient(180deg,#ffffff_0%,var(--accent-soft)_100%)] bg-clip-text text-[17px] font-bold leading-none tracking-[-0.055em] text-transparent transition-[filter] duration-[160ms] ease-out [-webkit-text-fill-color:transparent] group-hover:[filter:drop-shadow(0_0_8px_rgba(var(--accent-rgb),0.7))] max-[560px]:text-[16px]"
              style={titlebarNameStyle}
            >
              Nocturn
            </span>
          </button>
          <span
            className="whitespace-nowrap self-end pb-[2px] text-[11px] font-medium tracking-[0.04em] text-[rgba(241,245,249,0.72)]"
            style={monoTextStyle}
          >
            {appVersion}
          </span>
        </div>

        <div className="flex items-center gap-2 max-[560px]:gap-[6px] [-webkit-app-region:no-drag] [app-region:no-drag]">
          {shouldShowUpdateButton && (
            <Tooltip
              side="bottom"
              title={updateTooltipTitle}
              description={updateTooltipDescription}
            >
              <button
                type="button"
                className={cn(
                  toolbarButtonClass,
                  isUpdateBusy
                    ? updateToolbarBusyClass
                    : `${updateToolbarButtonClass} animate-[update-glow-pulse_2.2s_ease-in-out_infinite]`,
                )}
                onClick={() => void installUpdate()}
                aria-label={isUpdateBusy ? "Installing update" : "Install available update"}
                disabled={isUpdateBusy}
              >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <path d="M12 4v10" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" />
                  <path d="M8 10l4 4 4-4" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" />
                  <path d="M5 18h14" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" />
                </svg>
              </button>
            </Tooltip>
          )}

          <div className="flex gap-[3px] [-webkit-app-region:no-drag] [app-region:no-drag]">
            <button
              type="button"
              className={windowControlClass}
              onClick={() => void invoke("hide_window")}
              aria-label="Minimize to tray"
            >
              <svg width="10" height="2" viewBox="0 0 10 2" fill="none" aria-hidden="true">
                <rect width="10" height="1.5" rx="0.75" fill="currentColor" />
              </svg>
            </button>
            <button
              type="button"
              className={cn(windowControlClass, "hover:bg-[rgba(239,68,68,0.22)] hover:text-[#fca5a5]")}
              onClick={() => void invoke("close_app")}
              aria-label="Close"
            >
              <svg width="9" height="9" viewBox="0 0 9 9" fill="none" aria-hidden="true">
                <path d="M1 1l7 7M8 1l-7 7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              </svg>
            </button>
          </div>
        </div>
      </header>

      <main className="min-h-0 flex-1 overflow-hidden px-4 pb-3 pt-[14px] max-[560px]:px-3 max-[560px]:pb-[10px] max-[560px]:pt-3">
        <div
          key={activeView}
          className={cn(
            "h-full min-h-0",
            activeView === "settings"
              ? "animate-[view-in-right_220ms_ease-out]"
              : "animate-[view-in-left_220ms_ease-out]",
            "motion-reduce:animate-none",
          )}
        >
        {activeView === "settings" ? (
          <SettingsPage
            displays={displays}
            shortcutSettings={shortcutSettings}
            allowCursorExitActiveDisplays={allowCursorExitActiveDisplays}
            showOverlayHiddenApps={showOverlayHiddenApps}
            isMutating={isMutating}
            onUpdateShortcutSettings={setShortcutSettings}
            onToggleAllowCursorExitActiveDisplays={(allowed) => void setAllowCursorExitActiveDisplays(allowed)}
            onToggleShowOverlayHiddenApps={(enabled) => void setShowOverlayHiddenApps(enabled)}
            onResetToDefaults={() => void resetToDefaults()}
          />
        ) : (
          <DisplayLayout
            displays={displays}
            headline={displayHeadline}
            isMutating={isMutating}
            isLoading={isLoading}
            loadError={loadError}
            pendingDisplayIds={pendingDisplayIds}
            lastActiveDisplayId={lastActiveDisplayId}
            focusModeHotkey={shortcutSettings.focusModeHotkey}
            onFocusMode={() => void focusPrimary()}
            onToggle={(id) => void toggleDisplay(id)}
            onRetry={() => void loadDisplays()}
          />
        )}
        </div>
      </main>

      {mutationError && (
        <div
          className={cn(
            "absolute inset-x-3 z-30 flex items-center justify-between gap-3 rounded-[10px] border border-[rgba(248,113,113,0.35)] bg-[rgba(127,29,29,0.92)] px-3 py-2 text-[12px] text-[#fecaca] shadow-[0_10px_24px_rgba(2,6,23,0.35)]",
            activeView === "settings" ? "bottom-3" : "bottom-[86px]",
          )}
          role="alert"
        >
          <span className="truncate">{mutationError}</span>
          <button
            type="button"
            onClick={clearMutationError}
            className="shrink-0 rounded-[6px] px-2 py-1 text-[11px] font-semibold text-[#fecaca] transition-colors duration-[120ms] hover:bg-[rgba(255,255,255,0.08)]"
            aria-label="Dismiss error"
          >
            Dismiss
          </button>
        </div>
      )}

      {activeView !== "settings" && (
        <div className="shrink-0 border-t border-[var(--border)] px-3 pb-3 pt-[10px]">
          <button
            type="button"
            className={wakeButtonClass(hasHiddenDisplays)}
            onClick={() => void restoreAllDisplays()}
            disabled={!hasHiddenDisplays || isRestoreAllBusy}
            aria-label="Restore all blacked-out displays"
          >
            <span className="text-[13px] font-semibold leading-[1.05] tracking-[-0.03em]">Restore all displays</span>
          </button>
        </div>
      )}
    </div>
  );
}

export default App;
