type SettingsPageProps = {
  allowCursorExitActiveDisplays: boolean;
  isMutating: boolean;
  onToggleAllowCursorExitActiveDisplays: (allowed: boolean) => void;
};

export function SettingsPage({
  allowCursorExitActiveDisplays,
  isMutating,
  onToggleAllowCursorExitActiveDisplays,
}: SettingsPageProps) {
  return (
    <section className="settings-page" aria-label="Settings">
      <header className="settings-header">
        <h1 className="settings-title">Settings</h1>
      </header>

      <div className="settings-single-column">
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
