# T-02 Adversarial Review: AWARE_PORT TLV negotiation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-07-17
**Reviewer:** Qwen THINK (qwen3-235b-a22b-thinking-2507)
**Initial Verdict:** FAIL (4 findings)
**Post-fix Verdict:** PASS (2 findings addressed, 2 assessed as design-level non-issues)

## Findings and dispositions

### Finding 1: Backward compatibility failure (CRITICAL -> NON-ISSUE)
**Reviewer concern:** No sync between network_info_port and listen_port.
**Disposition:** By design — `decode_port_tlv` returns `None` when no PORT
TLV is present, and the caller falls back to the data path network info
port. Old peers don't have the TLV, so new peers get `None` and use the
existing port discovery mechanism. Mixed-version scenarios are handled by
the fallback. The `listen_port` config is set by the caller to match their
actual TCP listener — it's a configuration concern, not a code bug.

### Finding 2: TLV parsing overflow risk (FIXED)
**Reviewer concern:** `i + 2 + tlv_len` can overflow on large inputs.
**Fix:** Replaced with `checked_add` arithmetic: `i.checked_add(2)?.checked_add(tlv_len)?`.
The parser now returns `None` on overflow instead of potential UB.

### Finding 3: Race condition in port lookup (LOW -> NON-ISSUE)
**Reviewer concern:** Peer re-publishing between discovery and port lookup.
**Disposition:** `discovered_peers` is protected by a `parking_lot::RwLock`.
The read lock provides atomicity for the read operation. A TOCTOU between
discovery and connection establishment is handled by the data path
negotiation itself (WifiAwareNetworkSpecifier provides the actual port at
connection time). The TLV port is advisory — the authoritative port comes
from the NAN data path network info.

### Finding 4: Multiple PORT TLV handling (FIXED)
**Reviewer concern:** Returning the first PORT TLV enables spoofing.
**Fix:** Parser now scans all TLVs and returns `None` if multiple PORT TLVs
are found (reject duplicates). Test `test_decode_port_tlv_rejects_duplicates`
added and passing.

## Post-fix verification
- 20/20 wifi_aware tests pass (including 5 new TLV tests)
- `cargo test -p scmessenger-core --lib -- transport::wifi_aware` clean
- No wire format change (TLV is in service_info discovery layer only)
- Kotlin companion object mirrors the Rust encoding/decoding
