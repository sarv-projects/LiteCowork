export type ProbedProfile = { displayName: string };

/**
 * The Agents heading names only harnesses that have a probed profile on this Runtime.
 * It never lists a harness name from a static roster (011-T09, AC-019).
 */
export function agentSettingsDescription(profiles: readonly ProbedProfile[]): string {
  const names = [...new Set(profiles.map(profile => profile.displayName.trim()).filter(name => name.length > 0))];
  if (names.length === 0) {
    return "No agent harness has been probed on this Runtime yet. Probe a harness below to see its status. LiteCowork coordinates native harnesses without imposing an external reasoning loop.";
  }
  return `Configure probed native agent harnesses: ${names.join(", ")}. LiteCowork coordinates native harnesses without imposing an external reasoning loop.`;
}
