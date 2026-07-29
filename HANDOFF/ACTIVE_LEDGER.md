# ACTIVE_LEDGER

## Repo State
As of 2026-07-29

## Remaining LoC Estimate
0 LoC (doc only)

## Priority Queue

| Wave | ID     | Status    | Tier   | LoC | Notes |
|------|--------|-----------|--------|-----|-------|
| A    | A-01   | OPEN      | FLASH  | 0   |       |
| A    | A-02   | OPEN      | FLASH  | 0   |       |
| A    | A-03   | OPEN      | FLASH  | 0   |       |
| A    | A-05   | OPEN      | FLASH  | 0   |       |
| A    | A-06   | OPEN      | FLASH  | 0   |       |
| B    | B-01   | BLOCKED   | FLASH  | 0   | Blocked on E-01c |
| B    | B-02   | BLOCKED   | FLASH  | 0   | Blocked on E-01c |
| B    | B-03   | BLOCKED   | FLASH  | 0   | Blocked on E-01c |
| B    | B-04   | BLOCKED   | FLASH  | 0   | Blocked on E-01c |
| B    | B-05   | BLOCKED   | FLASH  | 0   | Blocked on E-01c |
| C    | C-01   | OPEN      | FLASH  | 0   |       |
| C    | C-02   | BLOCKED   | FLASH  | 0   | Requires H-03 sign-off |
| C    | C-03   | BLOCKED   | FLASH  | 0   | Requires H-03 sign-off |
| C    | C-04   | BLOCKED   | FLASH  | 0   | Requires H-03 sign-off |
| C    | C-05   | OPEN      | FLASH  | 0   |       |
| C    | C-06   | OPEN      | FLASH  | 0   |       |
| D    | D-01   | OPEN      | FLASH  | 0   |       |
| D    | D-02   | OPEN      | FLASH  | 0   |       |
| D    | D-03   | OPEN      | FLASH  | 0   |       |
| D    | D-05   | DONE      | FLASH  | 0   | Unwrap panic hardening |
| D    | D-06   | OPEN      | FLASH  | 0   |       |
| E    | E-01a  | DONE      | FLASH  | 0   | Constraints analysis |
| E    | E-01b  | BLOCKED   | FLASH  | 0   | Operator review needed |
| E    | E-01c  | BLOCKED   | FLASH  | 0   | Operator review needed |
| E    | E-02   | OPEN      | FLASH  | 0   |       |
| E    | E-04   | OPEN      | FLASH  | 0   |       |
| H    | H-01   | BLOCKED   | FLASH  | 0   | Human gates |
| H    | H-02   | BLOCKED   | FLASH  | 0   | Human gates |
| H    | H-03   | BLOCKED   | FLASH  | 0   | Human gates |
| H    | H-04   | BLOCKED   | FLASH  | 0   | Human gates |
| H    | H-05   | BLOCKED   | FLASH  | 0   | Human gates |
| T    | T-02   | OPEN      | FLASH  | 0   |       |
| T    | T-03   | OPEN      | FLASH  | 0   |       |
| T    | T-05   | OPEN      | FLASH  | 0   |       |
| T    | T-06   | DONE      | FLASH  | 0   | V1 known limitations |
| W    | W-01   | BLOCKED   | FLASH  | 0   | Frozen on E-01c |
| W    | W-02   | BLOCKED   | FLASH  | 0   | Frozen on E-01c |
| W    | W-03   | BLOCKED   | FLASH  | 0   | Frozen on E-01c |
| W    | W-04   | BLOCKED   | FLASH  | 0   | Frozen on E-01c |
| W    | W-05   | BLOCKED   | FLASH  | 0   | Frozen on E-01c |
| Z    | Z-02   | DONE      | FLASH  | 0   | Backlog tickets regenerated |
| Z    | Z-03   | DONE      | FLASH  | 0   | Active ledger updated |

## Rules

- Only one task may be IN_PROGRESS per lane
- Refer to dispatch ladder in [docs/ORCHESTRATION.md](docs/ORCHESTRATION.md) Section 2.1
- Security gates:
  - Adversarial on crypto/transport
  - Fusion Lite on WS-A

--- END FILE ---
