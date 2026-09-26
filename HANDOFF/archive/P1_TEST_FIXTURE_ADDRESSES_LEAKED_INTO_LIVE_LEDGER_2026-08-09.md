# P1 -- test-fixture addresses are in the live ledger and get dialled in production

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Open
Filed: 2026-08-09 (Windows lane, PR #139 CLI coordination run)
Severity: P1 -- wastes dial budget, and puts unsolicited connection attempts on
the public internet toward third-party infrastructure.

## Evidence

Captured from a running node at `b2953030`, debug transport filter on, in a
single short observation window:

```
29  /ip4/1.1.1.1/tcp/9000
25  /ip4/8.8.8.8/tcp/9000
24  /ip4/1.2.3.4/tcp/9000
77  /ip4/127.0.0.1/tcp/8080
62  /ip4/127.0.0.1/tcp/9002
42  /ip4/127.0.0.1/tcp/60636
34  /ip4/127.0.0.1/tcp/9090
34  /ip4/127.0.0.1/tcp/80
34  /ip4/127.0.0.1/tcp/443
```

`1.2.3.4`, `8.8.8.8` and `1.1.1.1` on port 9000 are not plausible SCMessenger
peers. `1.2.3.4` in particular is a canonical documentation/test address. These
read as test fixtures that were written into a real ledger and are now being
dialled by a production node.

The loopback block is a related but distinct problem: the node dials itself on
ports it has used, including `60636`, its own current external-address port.

Roughly 260 wasted dials in one window, on top of the self-dial and carrier-IPv6
waste already filed as
`P1_PROMISCUOUS_DIAL_WASTES_BUDGET_ON_SELF_AND_CELLULAR_2026-08-09.md`.

## Why this is separate from the dial-policy ticket

The promiscuous-dial ticket is about *prioritisation*: real candidates ranked
badly. This one is about *data hygiene*: entries that should never have been
persisted at all. A dial-policy fix that ranks LAN before carrier addresses will
still faithfully dial `1.2.3.4` once the LAN candidates are exhausted. They need
separate fixes.

## Two questions the fix must answer

1. **How did they get in?** Either a test helper wrote to the real ledger path
   instead of a temp dir, or a fixture list is compiled into a seed/bootstrap
   path that runs in release. Find the writer before adding a filter -- a filter
   alone leaves the source live and the next fixture will land the same way.
2. **What else is in there?** Audit the live ledger for other implausible
   entries rather than special-casing these three addresses.

## Acceptance criteria

1. Root cause identified: the code path that persisted these entries, cited by
   `file:line`.
2. That path can no longer write to a real ledger (test isolation, or removal of
   the seed data).
3. Ingress validation rejects documentation/reserved/special-use ranges before
   persisting: at minimum `1.2.3.4` and the RFC 5737 blocks
   (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`), plus a decision on
   well-known public resolvers.
4. A one-time cleanup for ledgers that already contain them -- the defect is
   persisted state, so fixing the code does not fix existing installs, including
   every device in the current fleet test.
5. Regression test that persists a fixture address and asserts it is rejected.

## Note on scope

Ledger ingress and dial candidate construction touch
`core/src/transport/`, which is merge-blocked under repo rule 8. This needs an
operator decision on direction and adversarial review before implementation, and
must not be handed to a worker to "just fix".
