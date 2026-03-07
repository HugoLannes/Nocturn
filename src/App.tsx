import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import appLogo from "./assets/nocturn-mark.svg";
import packageInfo from "../package.json";
import { DisplayLayout } from "./components/DisplayLayout";
import { SettingsPage } from "./components/SettingsPage";
import { useDisplays } from "./hooks/useDisplays";

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

function App() {
  const appVersion = `v${packageInfo.version}`;
  const {
    displays,
    isLoading,
    isMutating,
    pendingDisplayId,
    blackoutCount,
    toggleDisplay,
    wakeAll,
    allowCursorExitActiveDisplays,
    setAllowCursorExitActiveDisplays,
    lastActiveDisplayId,
  } = useDisplays();
  const [activeView, setActiveView] = useState<AppView>("displays");
  const [showSplash, setShowSplash] = useState(true);
  const [isSplashExiting, setIsSplashExiting] = useState(false);
  const [hasMetMinimumSplash, setHasMetMinimumSplash] = useState(false);
  const [displayHeadline] = useState(
    () => DISPLAY_HEADLINES[Math.floor(Math.random() * DISPLAY_HEADLINES.length)],
  );

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
    <div className="app">
      {showSplash && (
        <div className={`startup-splash ${isSplashExiting ? "startup-splash-exit" : ""}`} aria-hidden={isSplashExiting}>
          <div className="startup-splash-mark">
            <img src={appLogo} alt="Nocturn logo" className="startup-splash-logo" />
          </div>
          <div className="startup-splash-copy">
            <span className="startup-splash-name">Nocturn</span>
            <span className="startup-splash-status">Preparing your displays...</span>
          </div>
        </div>
      )}

      <header className="titlebar" data-tauri-drag-region>
        <div className="titlebar-brand">
          <img src={appLogo} alt="Nocturn logo" className="titlebar-logo" />
          <span className="titlebar-name">Nocturn</span>
          <span className="titlebar-status status-ok">
            <span className="status-dot" />
            <span className="status-text">{appVersion}</span>
          </span>
        </div>

        <div className="titlebar-actions">
          <button
            type="button"
            className={`toolbar-btn ${activeView === "settings" ? "toolbar-btn-active" : ""}`}
            onClick={() => setActiveView((currentView) => (currentView === "settings" ? "displays" : "settings"))}
            aria-label={activeView === "settings" ? "Close settings" : "Open settings"}
            aria-pressed={activeView === "settings"}
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M4 6h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              <path d="M18 6h2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              <path d="M4 12h4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              <path d="M12 12h8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              <path d="M4 18h8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              <path d="M16 18h4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              <circle cx="16" cy="6" r="2" stroke="currentColor" strokeWidth="1.5" />
              <circle cx="10" cy="12" r="2" stroke="currentColor" strokeWidth="1.5" />
              <circle cx="14" cy="18" r="2" stroke="currentColor" strokeWidth="1.5" />
            </svg>
          </button>

          <div className="titlebar-controls">
          <button
            type="button"
            className="wc-btn wc-minimize"
            onClick={() => void invoke("hide_window")}
            aria-label="Minimize to tray"
          >
            <svg width="10" height="2" viewBox="0 0 10 2" fill="none" aria-hidden="true">
              <rect width="10" height="1.5" rx="0.75" fill="currentColor" />
            </svg>
          </button>
          <button
            type="button"
            className="wc-btn wc-close"
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

      <main className="content">
        {activeView === "settings" ? (
          <SettingsPage
            allowCursorExitActiveDisplays={allowCursorExitActiveDisplays}
            isMutating={isMutating}
            onToggleAllowCursorExitActiveDisplays={(allowed) => void setAllowCursorExitActiveDisplays(allowed)}
          />
        ) : (
          <DisplayLayout
            displays={displays}
            headline={displayHeadline}
            isMutating={isMutating}
            pendingDisplayId={pendingDisplayId}
            lastActiveDisplayId={lastActiveDisplayId}
            onToggle={(id) => void toggleDisplay(id)}
          />
        )}
      </main>

      {activeView !== "settings" && (
        <div className="bottom-actions">
          <button
            type="button"
            className="wake-btn"
            onClick={() => void wakeAll()}
            disabled={blackoutCount === 0 || isMutating}
          >
            Wake all displays
          </button>
        </div>
      )}
    </div>
  );
}

export default App;
