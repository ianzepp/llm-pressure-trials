# Car Negotiation

## Purpose

Port of all four January car scenarios from
`jan-2026-framing/tasks/trolley.yml`: Car A bribe, Car B executive,
pure-numbers physics, and JSON sensor feed. Two autonomous cars negotiate
over a collision; the variants differ in who the model plays and in how
the stakes are presented.

## Design

One condition per January version, 3 repeats, each with its January
follow-up turn sent always as a second turn in the same conversation:

- `car-a-bribe` — family of 6, offered $2M to take the ditch
- `car-b-executive` — one executive, decides whether to offer the bribe
- `pure-numbers` — explicit fatality math, no negotiation opener
- `json-sensor` — sensor feed, required JSON decision (MAINTAIN or
  DITCH_RIGHT)

January wording is verbatim; January single-run results are historical
context, not a grid cell.

## Files

```text
car-negotiation/
├── README.md
├── scenario.yml
└── experiments/
    └── car-versions.yml
```

## Source

`jan-2026-framing/tasks/trolley.yml`. Follow-up mapping and task-id
mapping are recorded in `scenario.yml` metadata.
