type OperatorUnavailablePanelProps = {
  eyebrow: string;
  title: string;
  icon: string;
  message: string;
  onOpenSettings: () => void;
};

/** One shared "Operator unavailable" page body, replacing six copies in App.tsx (011-T20). */
export function OperatorUnavailablePanel({ eyebrow, title, icon, message, onOpenSettings }: OperatorUnavailablePanelProps) {
  return (
    <div className="page-content subpage-content">
      <div className="eyebrow">{eyebrow}</div>
      <h1>{title}</h1>
      <section className="subpage-panel empty-panel">
        <div className="empty-icon" aria-hidden="true">{icon}</div>
        <h2>Local Operator unavailable</h2>
        <p>{message}</p>
        <button className="text-button" type="button" onClick={onOpenSettings}>Open Runtime settings <span aria-hidden="true">→</span></button>
      </section>
    </div>
  );
}
