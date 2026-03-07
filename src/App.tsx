import { invoke } from "@tauri-apps/api/core";
import { DisplayCard } from "./components/DisplayCard";
import { useDisplays } from "./hooks/useDisplays";

function App() {
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
    lastActiveDisplayId,
  } = useDisplays();

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
      <header className="titlebar" data-tauri-drag-region>
        <div className="titlebar-brand">
          <span className="titlebar-icon">⚡</span>
          <span className="titlebar-name">Nocturn</span>
          <span className={`titlebar-status ${blackoutCount > 0 ? "status-alert" : "status-ok"}`}>
            <span className="status-dot" />
            <span className="status-text">
              {blackoutCount > 0
                ? `${blackoutCount} blacked out`
                : "Ready"}
            </span>
          </span>
        </div>

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
      </header>

      <main className="content">
        <div className="displays-list">
          {displays.map((display) => (
            <DisplayCard
              key={display.id}
              display={display}
              isMutating={isMutating}
              isPending={pendingDisplayId === display.id}
              isLastActiveDisplay={lastActiveDisplayId === display.id && !display.isBlackedOut}
              onToggle={(id) => void toggleDisplay(id)}
            />
          ))}
        </div>
      </main>

      <footer className="footer">
        <div className="terminal-panel" aria-live="polite">
          <span className="terminal-prompt">$</span>
          <span className={statusClassName}>{statusMessage ?? "System idle. Waiting for display action."}</span>
        </div>

        <button
          type="button"
          className="wake-btn"
          onClick={() => void wakeAll()}
          disabled={blackoutCount === 0 || isMutating}
        >
          Wake all displays
        </button>
      </footer>
    </div>
  );
}

export default App;
