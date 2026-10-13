import React from 'react';
import ReactDOM from 'react-dom/client';
import { invoke, isTauri } from '@tauri-apps/api/core';
import './style.css';

type Capability = { status: string; detail?: string };
type State = {
  phase: string;
  elapsed_ms: number;
  idle_ms: number | null;
  display_count: number;
  preview_active: boolean;
  preview_window_count: number;
  wake_count: number;
  capabilities: Record<string, Capability>;
  storage_notice: string | null;
  last_error: string | null;
};
const phaseLabels: Record<string, string> = {
  stopped: 'Ready when you are',
  working: 'Your session is running',
  paused: 'Your session is paused',
};
function App() {
  const [state, setState] = React.useState<State | null>(null);
  const [error, setError] = React.useState<string | null>(null);
  const [busy, setBusy] = React.useState(false);
  const preview = new URLSearchParams(location.search).has('break');
  const desktop = isTauri();
  const refresh = React.useCallback(async () => {
    if (!desktop) return;
    try {
      setState(await invoke<State>('proof_status'));
    } catch (e) {
      setError(String(e));
    }
  }, [desktop]);
  React.useEffect(() => {
    void refresh();
    const timer = setInterval(() => void refresh(), 1000);
    return () => clearInterval(timer);
  }, [refresh]);
  const command = React.useCallback(
    async (type: string) => {
      setBusy(true);
      setError(null);
      try {
        await invoke('dispatch', { command: { type } });
        await refresh();
      } catch (e) {
        setError(String(e));
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );
  React.useEffect(() => {
    if (!preview) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') void command('end_preview');
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [preview, command]);
  if (preview)
    return (
      <main className="break-screen">
        <div className="orbit" aria-hidden="true">
          ◎
        </div>
        <p className="eyebrow">A MOMENT FOR YOURSELF</p>
        <h1>
          Take a pause.
          <br />
          Breathe a little.
        </h1>
        <p>Find something in the distance. Let your eyes rest.</p>
        <button onClick={() => void command('end_preview')} disabled={busy}>
          Return to your screen <span>Esc</span>
        </button>
        <small>This preview closes automatically after 15 seconds.</small>
        <small>
          {state?.preview_window_count ?? '—'} display previews active
        </small>
        {error && <p role="alert">{error}</p>}
      </main>
    );
  const elapsed = Math.floor((state?.elapsed_ms ?? 0) / 1000);
  return (
    <main className="shell">
      <header>
        <div className="brand">
          <span className="brand-icon" aria-hidden="true">
            ◎
          </span>
          <span>AwayTime</span>
        </div>
        <span className="badge">PHASE 0 · INTEGRATION PROOF</span>
      </header>
      <section className="hero">
        <p className="eyebrow">MAKE SPACE FOR A PAUSE</p>
        <h1>{phaseLabels[state?.phase ?? 'stopped']}</h1>
        <p>A quieter rhythm for your screen time.</p>
        <div
          className="time"
          aria-label={`${Math.floor(elapsed / 60)} minutes ${elapsed % 60} seconds`}
        >
          {String(Math.floor(elapsed / 60)).padStart(2, '0')}
          <span>:</span>
          {String(elapsed % 60).padStart(2, '0')}
        </div>
        <p className="caption">
          Background session time · automatic breaks arrive in Phase 1
        </p>
        <div className="actions">
          <button
            disabled={!desktop || busy}
            onClick={() =>
              void command(
                state?.phase === 'working'
                  ? 'pause'
                  : state?.phase === 'paused'
                    ? 'resume'
                    : 'start',
              )
            }
          >
            {state?.phase === 'working'
              ? 'Pause session'
              : state?.phase === 'paused'
                ? 'Resume session'
                : 'Start session'}
          </button>
          <button
            className="secondary"
            disabled={!desktop || busy || !state || state.preview_active}
            onClick={() => void command('preview_break')}
          >
            Preview a break <span>↗</span>
          </button>
          <button
            className="text-button"
            disabled={!desktop || busy || state?.phase === 'stopped'}
            onClick={() => void command('stop')}
          >
            Stop
          </button>
        </div>
      </section>
      {!desktop && (
        <p className="notice">
          Open the desktop app to try tray controls, idle detection, and break
          windows. This browser view previews the interface.
        </p>
      )}
      {(error || state?.last_error) && (
        <p role="alert" className="notice error">
          {error ?? state?.last_error}
        </p>
      )}
      {state?.storage_notice && (
        <p className="notice">{state.storage_notice}</p>
      )}
      <section className="proof">
        <div className="section-title">
          <h2>Desktop integration</h2>
          <p>Live observations from this computer</p>
        </div>
        <div className="metrics">
          <article>
            <span>Connected displays</span>
            <strong>{state?.display_count ?? '—'}</strong>
            <small>Preview opens on every display</small>
          </article>
          <article>
            <span>Time without input</span>
            <strong>
              {state?.idle_ms == null
                ? '—'
                : `${Math.floor(state.idle_ms / 1000)}s`}
            </strong>
            <small>Activity observation only</small>
          </article>
          <article>
            <span>Wake observations</span>
            <strong>{state?.wake_count ?? '—'}</strong>
            <small>No automatic rest credit yet</small>
          </article>
        </div>
        <div className="capabilities">
          {Object.entries(state?.capabilities ?? {}).map(([name, c]) => (
            <div key={name}>
              <span>{name.replaceAll('_', ' ')}</span>
              <span
                className={c.status === 'supported' ? 'supported' : 'pending'}
                title={c.detail}
              >
                {c.status === 'supported'
                  ? 'Available'
                  : (c.detail ?? c.status.replaceAll('_', ' '))}
              </span>
            </div>
          ))}
        </div>
      </section>
      <footer>
        <span>Local data. No account required.</span>
        <span>Closing this window keeps your session running.</span>
      </footer>
    </main>
  );
}
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
