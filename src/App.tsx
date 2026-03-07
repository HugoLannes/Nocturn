import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import crescentLogo from "../crescent.png";
import { DisplayLayout } from "./components/DisplayLayout";
import { SettingsPage } from "./components/SettingsPage";
import { useDisplays } from "./hooks/useDisplays";

type AppView = "displays" | "settings";

function App() {
  const appVersion = "v0.0.0";
  const {
    displays,
    isLoading,
    error,
    feedback,
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

  const statusMessage = isLoading
    ? "Scanning displays..."
    : error
      ? error
      : feedback;
  const statusClassName = error
    ? "terminal-line state-error"
    : feedback
      ? "terminal-line state-feedback"
      : "terminal-line";

  return (
    <div className="app">
      {showSplash && (
        <div className={`startup-splash ${isSplashExiting ? "startup-splash-exit" : ""}`} aria-hidden={isSplashExiting}>
          <div className="startup-splash-mark">
            <img src={crescentLogo} alt="Nocturn logo" className="startup-splash-logo" />
          </div>
          <div className="startup-splash-copy">
            <span className="startup-splash-name">Nocturn</span>
            <span className="startup-splash-status">Preparing your displays...</span>
          </div>
        </div>
      )}

      <header className="titlebar" data-tauri-drag-region>
        <div className="titlebar-brand">
          <img src={crescentLogo} alt="Nocturn logo" className="titlebar-logo" />
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
              <path
                d="M10.325 4.317a1.724 1.724 0 0 1 3.35 0 1.724 1.724 0 0 0 2.573 1.066 1.724 1.724 0 0 1 2.898 1.674 1.724 1.724 0 0 0 .865 2.665 1.724 1.724 0 0 1 0 3.556 1.724 1.724 0 0 0-.865 2.665 1.724 1.724 0 0 1-2.898 1.674 1.724 1.724 0 0 0-2.573 1.066 1.724 1.724 0 0 1-3.35 0 1.724 1.724 0 0 0-2.573-1.066 1.724 1.724 0 0 1-2.898-1.674 1.724 1.724 0 0 0-.865-2.665 1.724 1.724 0 0 1 0-3.556 1.724 1.724 0 0 0 .865-2.665 1.724 1.724 0 0 1 2.898-1.674 1.724 1.724 0 0 0 2.573-1.066Z"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <path d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
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
            isMutating={isMutating}
            pendingDisplayId={pendingDisplayId}
            lastActiveDisplayId={lastActiveDisplayId}
            onToggle={(id) => void toggleDisplay(id)}
          />
        )}
      </main>

      <footer className="footer">
        <div className="terminal-panel" aria-live="polite">
          <span className="terminal-prompt">$</span>
          <span className={statusClassName}>
            {activeView === "settings"
              ? allowCursorExitActiveDisplays
                ? "The cursor can leave the active display area."
                : "The cursor is confined to the active display area."
              : statusMessage ?? "System idle. Waiting for display action."}
          </span>
        </div>

        {activeView !== "settings" && (
          <button
            type="button"
            className="wake-btn"
            onClick={() => void wakeAll()}
            disabled={blackoutCount === 0 || isMutating}
          >
            Wake all displays
          </button>
        )}
      </footer>
    </div>
  );
}

export default App;
