import { buildActivityCards, planChecklist, type ActivityEntry } from "./turn-activity";

/**
 * REPORTED activity for one turn attempt (001-T25). Cards are collapsed by default and say
 * the agent reported the activity; nothing here is a LiteCowork Effect, receipt or grant.
 */
export function TurnActivityView({ entries }: { entries: readonly ActivityEntry[] }) {
  const cards = buildActivityCards(entries);
  const plan = planChecklist(entries);
  if (cards.length === 0 && !plan) return null;
  return (
    <details className="turn-activity" data-testid="turn-activity">
      <summary>Activity reported by the agent ({cards.length})</summary>
      {plan && <p className="turn-activity-plan" role="note">{plan.label}</p>}
      <ul className="turn-activity-list">
        {cards.map(card => (
          <li key={card.key} className={`turn-activity-card ${card.state.toLowerCase()}`}>
            <strong>{card.title}</strong> <span>{card.summary}</span>
          </li>
        ))}
      </ul>
    </details>
  );
}
