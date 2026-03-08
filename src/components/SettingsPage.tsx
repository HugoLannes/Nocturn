type SettingsPageProps = {
  allowCursorExitActiveDisplays: boolean;
  showOverlayHiddenApps: boolean;
  isMutating: boolean;
  onToggleAllowCursorExitActiveDisplays: (allowed: boolean) => void;
  onToggleShowOverlayHiddenApps: (enabled: boolean) => void;
};

export function SettingsPage({
  allowCursorExitActiveDisplays,
  showOverlayHiddenApps,
  isMutating,
  onToggleAllowCursorExitActiveDisplays,
  onToggleShowOverlayHiddenApps,
}: SettingsPageProps) {
  return (
    <section className="settings-page" aria-label="Settings">
      <header className="settings-header">
        <span className="layout-eyebrow">Settings</span>
        <h1 className="layout-title">Set things your way.</h1>
      </header>

      <div className="settings-single-column">
        <article className="settings-card">
          <div className="settings-card-header">
            <div>
              <h2 className="settings-category">Overlays</h2>
            </div>
          </div>

          <label className="settings-toggle-row">
            <div>
              <span className="settings-control-label">Show hidden apps</span>
              <p className="settings-control-hint">
                Display the app names detected behind blackout overlays. Turn this off for a fully plain black screen.
              </p>
            </div>

            <button
              type="button"
              className={`settings-switch ${showOverlayHiddenApps ? "settings-switch-on" : ""}`}
              onClick={() => onToggleShowOverlayHiddenApps(!showOverlayHiddenApps)}
              aria-pressed={showOverlayHiddenApps}
              aria-label="Show hidden apps on blackout overlays"
              disabled={isMutating}
            >
              <span className="settings-switch-thumb" />
            </button>
          </label>
        </article>

        <article className="settings-card">
          <div className="settings-card-header">
            <div>
              <h2 className="settings-category">Cursor</h2>
            </div>
          </div>

          <label className="settings-toggle-row">
            <div>
              <span className="settings-control-label">Cursor freedom</span>
              <p className="settings-control-hint">
                Turn this on to let the mouse travel outside the active monitors. Turn it off to confine the pointer.
              </p>
            </div>

            <button
              type="button"
              className={`settings-switch ${allowCursorExitActiveDisplays ? "settings-switch-on" : ""}`}
              onClick={() => onToggleAllowCursorExitActiveDisplays(!allowCursorExitActiveDisplays)}
              aria-pressed={allowCursorExitActiveDisplays}
              aria-label="Allow mouse to leave active displays"
              disabled={isMutating}
            >
              <span className="settings-switch-thumb" />
            </button>
          </label>
        </article>
      </div>
    </section>
  );
}
