import { summarizeStopPreview, type StopPreviewState } from "./runtime-stop-state.ts";

type Props = {
  state: StopPreviewState;
  runtimeId: string | null;
  incarnationId: string | null;
};

/** Renders the stop preview result. It holds no buttons: the Preview control lives in SettingsView. */
export function RuntimeStopPreviewPanel({ state, runtimeId, incarnationId }: Props) {
  const summary = summarizeStopPreview(state, { runtimeId, incarnationId });
  return (
    <div className="runtime-stop-preview" aria-live="polite">
      <p role="status" data-stop-verdict={state.status === "READY" ? state.preview.verdict : state.status}>
        {summary.verdictLabel}
      </p>
      {summary.items.length > 0 && (
        <ul aria-label="Items still running or unsettled">
          {summary.items.map((item) => <li key={item.label}>{item.label}: {item.count}</li>)}
        </ul>
      )}
      {summary.reasons.length > 0 && (
        <ul aria-label="Why a stop or update is not safe">
          {summary.reasons.map((reason, index) => <li key={`${reason}-${index}`}>{reason}</li>)}
        </ul>
      )}
      <p data-stop-safely="unavailable">{summary.stopReason || "Stop safely is available."}</p>
      <p data-update-gate={summary.updateGate.allowed ? "allowed" : "refused"}>{summary.updateGate.message}</p>
    </div>
  );
}
