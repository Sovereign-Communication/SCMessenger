# V040 freebuff operator decisions — required before tag

Status: AWAITING OPERATOR — orchestrator cannot improvise these (AGENTS rule 9)
Date: 2026-09-20
Context: `HANDOFF/V040_TAG_PATH_UNIFIED_2026-09-20.md`

Return rulings by editing this file on a branch the orchestrator merges, or by
telling the orchestrator session to record them in the unified path §7.

---

## R1 — Release keystore / D2 (blocking public APK)

**Facts:** secrets exist in GitHub; keystore files at `C:\Users\SCM\kiee\`
(`scmessenger-release.jks`); Multi-Platform Release Pipeline last runs failed;
historical failure: `SCMESSENGER_KEY_ALIAS is not present in the decoded keystore`.

**Action (operator, interactive):**

```
scripts/verify_release_keystore.sh C:\Users\SCM\kiee\scmessenger-release.jks <alias>
```

Then ensure `SCMESSENGER_KEY_ALIAS` **value** matches the resolved alias
exactly (PKCS12 is case-sensitive). Re-run release rehearsal:

```
gh workflow run release.yml -f artifacts_only=true
```

**Ruling needed:** alias value confirmed? rehearsal green Y/N? run URL: ______

---

## R2 — SEC-03 sled / deny.toml

**Facts:** `deny.toml` still waives multiple RUSTSEC ids via sled with no
expiry. #305 RCA escalated; no dependency change made (rule 9).

**Pick one:**

- [ ] **(a) Accept for tag:** dated waiver, named owner, migration budget in
      0.5.0. Record expiry date: __________
- [ ] **(b) Migrate on a branch** after tag; do not block tag on completion.
      Candidate engine: __________

**Ruling:** (a) / (b) — ______

---

## R3 — AND-06 Kotlin curve math

**Facts:** UniFFI `is_valid_public_key` on main (#305, canonical decode fixed).
Kotlin still contains Ed25519 BigInteger math in `PeerIdValidator.kt` and
related ViewModels. Ticket `HANDOFF/todo/P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md`
still says must land before tag.

**Pick one:**

- [ ] **(A) Pre-tag:** freebuff implements Kotlin half; acceptance grep clean;
      wire via UniFFI; Rule-8 if surface changes.
- [ ] **(B) Tag with dated accept:** Kotlin half is 0.5.0; update ticket text
      so it stops contradicting the tag path.

**Ruling:** (A) / (B) — ______

---

## R4 — External crypto audit commission (G4-2)

Commissioning is the gate, not completion.

- Firm: ______
- Scope: ______
- Price: ______
- Dates: ______

---

## R5 — Optional waivers for the public alpha

- [ ] AWS IP-churn autonomous rediscovery: **known limitation** + manual
      bootstrap runbook (V050-B1/B2 remains open design work).
- [ ] BLE real-time chat + cellular-only bootstrap: **UNVERIFIED** on tag
      notes; operator-supervised retests post-tag.
- [ ] Security Scan schedule lane red: **not a D1 blocker** under current
      branch-protection contexts (operator confirms).

**Confirm each with initials/date:** ______

---

## R6 — Open PR batch (non-code judgement)

- [ ] CLOSE #215 (superseded routing wire-up; CONFLICTING)
- [ ] CLOSE #156 if Docker Integration Suite remains green on main
- [ ] Defer dependabot wave / Apple docs / identity-unification PRs to post-tag
- [ ] Pull #322/#325 into the tag candidate (orchestrator merges when green)

---

Recording: orchestrator will update `HANDOFF/V040_TAG_PATH_UNIFIED_2026-09-20.md`
§7 when these boxes are filled.
