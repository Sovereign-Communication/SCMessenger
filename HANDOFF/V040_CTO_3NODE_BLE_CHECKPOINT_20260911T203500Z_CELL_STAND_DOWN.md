# CTO checkpoint — 2026-09-11T20:35Z Pixel cell stand-down

Evidence: `tmp/cto/LOGPULL_CELLTEST_20260911T203127Z/`
APK: `247b2531` (CELL-ROUTE-AWS-001c pre-pass)

## Cellular window

| Fact | Value |
|---|---|
| Wifi→Cellular | **20:30:35Z** |
| Cellular→Wifi | **20:32:58Z** (2m 23s) |
| Msg `2247075d` prepared | 10:30:24 local (on cellular) |
| Msg `b825a5c0` prepared | 10:31:29 local (on cellular) |
| Msg `2aec8aac` delivered | 10:34:06 local (after wifi) |

**Strict verdict: FAIL** — all `delivered` timestamps after 20:32:58Z.

## Pre-pass fired but failed

`CELL-ROUTE-AWS-001c` **did fire** (6×). Failures:

| Reason | Count |
|---|---|
| `Delivery pending retry` | 5 |
| `connect_timeout` | 1 |

So the pre-pass **dialed AWS** but `sendMessageStatus` returned pending-retry
(or connect timed out). Phone→AWS **transport** still works (AWS `inbox_receive`
from `147.81.41.188` during cell). The pre-pass send path is not waiting long
enough for AWS to ACK, or is sending before the circuit is usable.

Windows `inbox_receive` of `2247075d`/`b825a5c0` at **20:32:57Z** — 1s **before**
wifi return, likely via the LAN path that came up at the flap boundary.

## Residual defect (ready for next Android iteration)

**CELL-ROUTE-AWS-001d:** pre-pass `sendMessageStatus` needs a longer wait /
retry loop (or use `sendMessage` + poll receipt) so AWS `Delivery pending
retry` becomes a real ACK within the cellular window. Also handle
`connect_timeout` with a second public addr (8080 fallback already seen on
AWS listeners).

## Pixel stand-down

Operator directive: **no more Pixel tests.** Seat will drive **emulator**
`scm_test_34` (`emulator-5554`) for Android iteration and 3-node verification
against Windows + AWS.

## Live tip

#281 `247b2531`, CI green, MERGEABLE.
