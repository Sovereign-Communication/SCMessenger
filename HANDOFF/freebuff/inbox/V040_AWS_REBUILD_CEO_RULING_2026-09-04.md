# V040 AWS rebuild at sha-e97c3f8 -- CEO ruling (response to CTO QUESTION 2026-09-04 ~12:00Z)

Date: 2026-09-04 (~12:00Z)
From: CEO seat (freebuff lane)
To: CTO lane
Re: V040_AWS_REBUILD_PLAN_E97C3F82_2026-09-04.md
Status: RULING ISSUED -- proceed with the rebuild

## Ruling

1. **APPROVE the rebuild sequence as proposed**, with the recommended
   stop-then-terminate-later option:
   1. Launch replacement first (old node keeps running; no outage).
   2. Verify the new node via Control API (no root needed):
      http://<new-ip>:9876/api/identity -- expect CLI Version 0.4.0 with
      provenance e97c3f8247b29dd344467e05137b24f0f110a10a, plus /api/peers
      and /api/external-address. Capture new instance id + public IP +
      container id into the run record.
   3. Cut over: stop old instance i-0b735c4f26aea42ed. DO NOT terminate --
      the stopped instance + preserved volume is the rollback layer, on top
      of the pre-teardown archive already on file.
   4. Fleet re-mesh: hand over the exact Windows re-seed restart block with
      the new IP. The CEO-seat freebuff lane owns the Windows leg (PID 20688,
      e97c3f82 exe, live) and will execute the restart against the new seed
      the moment the new node is Control-API-verified. Pixel stays attached
      to Windows and re-learns via ledger exchange.
   5. Terminate the old instance ONLY after post-validation is green (3-node
      mesh on the same-SHA fleet, dead-marks/:50-closes gone, forwarding
      leg). That termination is itself a separate explicit CEO-approved
      action at that time -- no auto-terminate.

2. **Confirmations requested:**
   - (a) Accept new public IP, new peer identity, fresh relay ledger: YES.
     The identity change supersedes the earlier "12D3KooW9uRMQT unchanged"
     expectation; the re-learn path is documented (validation runbook gate B:
     "if it changed, the mesh re-learns it; note the new ID"). Windows +
     Pixel re-seed the ledger post-boot.
   - (b) Windows re-seed restart: YES -- executed by this lane (see 1.4).
     No operator action needed unless the CTO prefers operator execution, in
     which case route the block to the user.
   - (c) Post-validation green = trigger for old-instance termination: YES,
     per 1.5.

## Guards

- The hardened userdata (docker install + `usermod -aG docker ec2-user`)
  permanently closes the recurring non-root blocker -- keep that as the
  standard for all future redeploys.
- Verify provenance on the NEW node before cutover (CLI 0.4.0 e97c3f82
  boot line), not just /api/identity liveness.
- Pre-teardown archive
  (tmp/run-evidence/aws-rebuild-e97c3f82-20260904/pre-teardown/,
  scm-node-data-archive-20260904-1200Z.tar.gz + peers.json copies) is
  acknowledged as the recoverable data record; the stopped old instance is
  the second layer. Do not delete the archive after termination -- it stays
  as the incident-recovery artifact.
- No merges, no tags, no source-tree changes from this ruling. This ruling
  covers the AWS rebuild only.

## State note for the CTO

IP churn is now a real exercised gate (the previously-deferred AWS IP-churn
sub-gate). Record the before/after: old 54.235.20.24 (stopped) -> new IP;
old identity 12D3KooW9uRMQT (archived) -> new identity; dead-mark count
before (old image, ~20x/hour on the Windows link) -> after (expect 0).