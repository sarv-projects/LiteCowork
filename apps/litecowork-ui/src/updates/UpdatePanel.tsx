import { useEffect, useMemo, useReducer, useRef } from "react";
import {
  canCheck,
  createUpdateController,
  describeUpdateState,
  initialUpdateState,
  installAvailability,
  updateReducer,
  type UpdateApi,
  type UpdatePlatform,
} from "./update-state.ts";

type UpdatePanelProps = {
  api: UpdateApi;
  platform: UpdatePlatform;
};

/** Hosted by the Feature 011 Settings > Advanced tab. Renders gate results; computes nothing itself. */
export function UpdatePanel({ api, platform }: UpdatePanelProps) {
  const [state, dispatch] = useReducer(updateReducer, platform, initialUpdateState);
  const stateRef = useRef(state);
  stateRef.current = state;
  const messageRef = useRef<HTMLParagraphElement>(null);
  const announced = useRef(state.status);
  const controller = useMemo(() => createUpdateController({
    api,
    dispatch,
    getState: () => stateRef.current,
    newRequestId: () => crypto.randomUUID(),
    isOffline: () => typeof navigator !== "undefined" && navigator.onLine === false,
  }), [api]);

  // Move focus to the result message after each command settles into a new state.
  useEffect(() => {
    if (announced.current !== state.status) {
      announced.current = state.status;
      if (state.status !== "CHECKING" && state.status !== "GATING") messageRef.current?.focus();
    }
  }, [state.status]);

  const install = installAvailability(state);
  const busy = state.status === "CHECKING" || state.status === "GATING" || state.status === "INSTALLING";
  const installLabel = state.status === "REFUSED_NOT_QUIESCENT" || state.status === "FAILED" ? "Retry install" : "Install update";

  return (
    <section className="update-panel" aria-labelledby="update-panel-title" aria-busy={busy}>
      <h3 id="update-panel-title">Updates</h3>
      <p
        ref={messageRef}
        tabIndex={-1}
        role="status"
        aria-live="polite"
        data-update-state={state.status}
        className="update-panel-message"
      >
        {describeUpdateState(state)}
      </p>
      {state.status === "REFUSED_NOT_QUIESCENT" && state.reasons.length > 0 && (
        <ul aria-label="Items still running or unknown">
          {state.reasons.map((reason, index) => (
            <li key={`${reason.label}-${index}`}>{reason.label} ({reason.status})</li>
          ))}
        </ul>
      )}
      <div className="update-panel-actions">
        <button
          type="button"
          className="secondary-button"
          disabled={!canCheck(state)}
          onClick={() => { void controller.check(); }}
        >
          {state.status === "CHECK_FAILED" || state.status === "OFFLINE" ? "Retry check" : "Check for updates"}
        </button>
        <button
          type="button"
          className="primary-button"
          disabled={!install.enabled}
          aria-describedby={install.enabled ? undefined : "update-install-reason"}
          onClick={() => { void controller.install(); }}
        >
          {installLabel}
        </button>
      </div>
      {!install.enabled && (
        <p id="update-install-reason" className="update-panel-reason">{install.reason}</p>
      )}
    </section>
  );
}
