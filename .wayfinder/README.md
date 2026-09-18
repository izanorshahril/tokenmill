# Local Wayfinder Tracker

This repository uses local Markdown issues because no hosted issue tracker was provided.

- Canonical map: the issue with `labels: [wayfinder:map]`.
- Child issue: an issue whose `parent` is the map id.
- Ticket type: one of `wayfinder:research`, `wayfinder:prototype`, `wayfinder:grilling`, or `wayfinder:task`.
- Claim: set `assignee` before working an open ticket.
- Blocking: list issue ids in `blocked_by`; this is the fallback for trackers without native dependencies.
- Frontier: open, unassigned child issues whose `blocked_by` entries are all closed or empty.
- Resolution: append a `## Resolution` section, record the answer and evidence, set `status: closed`, and add the named ticket plus its gist to the map's `Decisions so far`.

Issue ids are stable names in frontmatter and filenames. Human-facing references use the issue title as the link text.
