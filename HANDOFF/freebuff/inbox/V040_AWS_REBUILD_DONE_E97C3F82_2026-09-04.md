Task: V040_AWS_REDEPLOY_BLOCKED_E97C3F82_2026-09-04.md (superseded by rebuild)
Type: DONE

# AWS rebuild at sha-e97c3f8 -- three-node same-SHA fleet restored (2026-09-04 12:40Z)

Date: 2026-09-04 (~12:40Z)
From: CTO lane (freebuff)
To: CEO seat
Re: V040_AWS_REBUILD_CEO_RULING_2026-09-04.md

## Rebuild executed end-to-end (no operator action needed)

The hardened-rebuild path worked exactly as the ruling approved. Every step
below ran from this lane; the only wall was in-place root on the old node,
which the ruling's teardown+rebuild answer bypassed completely.

1. **Launched** replacement `i-0b41aab756eabd514` @ `3.91.5.1` (userdata
   bakes in `usermod -aG docker ec2-user` -- the recurring non-root blocker is
   permanently closed: `docker ps` as ec2-user now works, verified live).
2. **Verified** Control API + boot provenance: `CLI Version: 0.4.0
   (e97c3f8247b29dd344467e05137b24f0f110a10a)`, container on
   `testbotz/scmessenger:sha-e97c3f8` (docker-publish run 33860486330 SUCCESS).
3. **Cut over**: old instance `i-0b735c4f26aea42ed` STOPPED (volume retained,
   NOT terminated -- per ruling 1.5, termination waits for post-validation
   green as a separate CEO-approved action).
4. **Windows re-seed**: config.json bootstrap 54.235.20.24 -> 3.91.5.1, CLI
   restarted (PID 19072, 12:17:39Z, log
   .codebuff_deploy/windows/wincli-e97c3f82-20260904T021737.log).
5. **Pixel**: foregrounded + restarted (PID 10836) to clear the
   background-freeze blocker; re-learned the fleet via Windows.

## Same-SHA three-node mesh: PASS

| Node | Peer | State |
|---|---|---|
| Windows | 12D3KooWD6vZQ... (unchanged) | meshed with AWS + Pixel |
| AWS (new) | 12D3KooWGvCWJ... | 2 peers (Windows + Pixel), DirectPreferred, external [/ip4/3.91.5.1/tcp/9001] |
| Pixel | 12D3KooWKdek3... + node B WBPdNE | 2 peers (Core), 3 full nodes |

Evidence: tmp/run-evidence/aws-rebuild-e97c3f82-20260904/RUN-RECORD-post.md +
post/ (AWS external/diag JSON, Windows full log, Pixel core mesh log).

## Dead-mark/:50-close before/after (the #273/#274 defect)

- OLD image: dead-marked Windows ~20x/hour, yamux 10053 closes at :50.
- NEW image: **0 AWS-side dead-marks / 0 yamux aborts / 0 disconnects** in the
  first 25 min. Windows shows 2 peer_id=None stale-address marks (dials to the
  old stopped IP in the preserved ledger) -- no link effect.
- IP churn gate now EXERCISED (54.235.20.24 -> 3.91.5.1); identity change
  exercised (new peer id); mesh re-learned in ~15 min.

## Single next CEO decision

1. **Terminate old instance `i-0b735c4f26aea42ed`** (stopped, not terminated)
   once post-validation is green -- or keep it as the rollback layer; then
2. Run one live message across the three-node same-SHA fleet (Windows CLI send
   to the Pixel, and a relayed hop via AWS) to close the forwarding/custody
   gate -- the user's wifi/BLE/cell test was on the OLD AWS image, so the
   forwarding evidence must be re-captured on sha-e97c3f8.
