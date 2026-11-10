import { useEffect, useMemo, useState } from "react";
import {
  WIDGET_IDS,
  WIDGET_TITLES,
  loadPins,
  savePins,
  togglePin,
  type KeyValueStorage,
  type WidgetId,
  type WidgetState,
} from "./dashboard-state.ts";

export type WidgetSource = { load: () => Promise<WidgetState> } | { unavailable: string };

type Props = {
  workspaceId: string;
  storage: KeyValueStorage;
  sources: Record<WidgetId, WidgetSource>;
};

export function WidgetBody({ state }: { state: WidgetState }) {
  if (state.status === "LOADING") return <p role="status">Loading…</p>;
  if (state.status === "UNAVAILABLE") {
    return <p role="status" data-widget-state="UNAVAILABLE">Unavailable: {state.reason}</p>;
  }
  return (
    <>
      {state.lines.length === 0 ? <p data-widget-state="READY">None reported by the Runtime.</p> : (
        <ul data-widget-state="READY">
          {state.lines.map((line) => <li key={line.label}>{line.label}: {line.value}</li>)}
        </ul>
      )}
      {state.note && <p className="dashboard-note">{state.note}</p>}
    </>
  );
}

/** Persistent Workspace Dashboard. Pin state is per Workspace and stored locally. */
export function WorkspaceDashboard({ workspaceId, storage, sources }: Props) {
  const [pins, setPins] = useState<WidgetId[]>(() => loadPins(storage, workspaceId));
  const [states, setStates] = useState<Partial<Record<WidgetId, WidgetState>>>({});

  useEffect(() => {
    setPins(loadPins(storage, workspaceId));
    setStates({});
  }, [storage, workspaceId]);

  const pinned = useMemo(() => new Set(pins), [pins]);

  useEffect(() => {
    let current = true;
    for (const id of WIDGET_IDS) {
      if (!pinned.has(id)) continue;
      const source = sources[id];
      if ("unavailable" in source) {
        setStates((previous) => ({ ...previous, [id]: { status: "UNAVAILABLE", reason: source.unavailable } }));
        continue;
      }
      setStates((previous) => ({ ...previous, [id]: { status: "LOADING" } }));
      source.load().then(
        (state) => { if (current) setStates((previous) => ({ ...previous, [id]: state })); },
        () => { if (current) setStates((previous) => ({ ...previous, [id]: { status: "UNAVAILABLE", reason: "The Runtime did not return this projection." } })); },
      );
    }
    return () => { current = false; };
  }, [pinned, sources, workspaceId]);

  const toggle = (id: WidgetId) => {
    const next = togglePin(pins, id);
    setPins(next);
    savePins(storage, workspaceId, next);
  };

  return (
    <div className="page-content subpage-content workspace-dashboard">
      <div className="eyebrow">WORKSPACE</div>
      <h1>Dashboard</h1>
      <div className="dashboard-grid">
        {WIDGET_IDS.map((id) => (
          <section key={id} className="subpage-panel dashboard-widget" aria-labelledby={`dashboard-${id}-title`} data-widget={id} data-pinned={pinned.has(id)}>
            <div className="dashboard-widget-header">
              <h2 id={`dashboard-${id}-title`}>{WIDGET_TITLES[id]}</h2>
              <button type="button" className="text-button" aria-pressed={pinned.has(id)} onClick={() => toggle(id)}>
                {pinned.has(id) ? "Unpin" : "Pin"}
              </button>
            </div>
            {pinned.has(id)
              ? <WidgetBody state={states[id] ?? { status: "LOADING" }} />
              : <p className="dashboard-note">Not pinned.</p>}
          </section>
        ))}
      </div>
    </div>
  );
}
