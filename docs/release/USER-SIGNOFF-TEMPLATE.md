# USER sign-off record (template)

Copy this into the bundle `user_signoff` object; do not commit a filled copy with user content. A journey without an ACCEPT decision stays below QUALIFIED in the release matrix (AC-014).

| Field | Value |
|---|---|
| Row id | |
| Journey id | |
| Bundle SHA-256 | |
| Reviewer | |
| Date (YYYY-MM-DD) | |
| Decision | ACCEPT, PARTIAL or REJECT |
| Blocked or unknown items | |

**Not a green row:** a PARTIAL or REJECT decision, or any blocked or unknown item, means this journey is not a green row for the release. State exactly what was not observed (for example "screen reader walkthrough not performed").

Never paste credentials, tokens, private prompts or unredacted user content here.
