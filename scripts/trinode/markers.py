"""Single source of truth for every log marker the verifier understands.

Every row was derived from source on origin/main (commit 3b4177be7) with
`git grep`; the `src` field is the file:line of the emitting statement so a
marker drift is a one-grep fix. Rows marked `src="NOT-IN-CODE"` are matched
only so that a future emitter (see docs/runbooks/TRI_NODE_VERIFY.md gap list)
is picked up automatically; today they never fire.

Evidence standard (mandatory, see CLAUDE.md fleet-run scoring): delivery is
scored ONLY on receiver-side decrypt + durable history write + delivery
receipt. `evidence=False` rows (transport ACKs, local acceptance) are parsed
and shown but are never counted toward VERIFIED.

Event vocabulary used by the correlator:
  send_prepared        sender wrote the outbound message to local history
  delivery_state       sender-side state transition (carries state=...)
  rx_decrypted         receiver decrypted an inbound message
  rx_history           receiver durably holds the message in history
  receipt_sent         receiver emitted the delivery receipt
  receipt_received     sender received the delivery receipt
  history_delivered    sender's history record flipped to delivered
  transport_ack        transport-level ACK (NOT evidence)
  peer_identified      libp2p identify completed with a peer
  connection_established  transport connection up (peer = id prefix)
  bootstrap_dial       node dialed a bootstrap/seed address
  seed_dial            CLI boot seed-dial sweep outcome
  invite_imported      join-bundle / invite seeds imported
  ledger_*             ledger exchange lifecycle (see ledger section)
  custody_audit        relay custody audit counter line
  routing_peer_seen    [ROUTING] peer_seen (routing engine fed a sighting)
  transport_status     [TRANSPORT] kind/state/detail (+ peers=N periodic)
  rx_drop              [RX-DROP] receiver dropped a message (stage/reason)
  rx_drop_suppressed   [RX-DROP] suppressed=<n> stage=<s> (#512)
  rx_stall             [RX-STALL] receive-path stall (#512)
  mesh_stop            [MESH-STOP] stop sequence phases
"""
from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Pattern, Tuple


@dataclass(frozen=True)
class Marker:
    name: str                      # stable id, used in docs and tests
    event: str                     # normalized event name
    pattern: Pattern[str]          # searched against the message text
    src: str                       # file:line of the emitter on origin/main
    evidence: bool = True          # False => parsed but never scores delivery
    inferred: bool = False         # True => implied by code path, not explicit
    extra_events: Tuple[str, ...] = ()   # additional events emitted per match
    note: str = ""


def _m(name, event, rx, src, **kw) -> Marker:
    return Marker(name, event, re.compile(rx), src, **kw)


MARKERS: List[Marker] = [
    # ---- message lifecycle: Android (Timber) -------------------------------
    _m("android_delivery_state", "delivery_state",
       r"delivery_state msg=(?P<msg>\S+) state=(?P<state>[a-z_]+)(?: detail=(?P<detail>.*))?$",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:11996",
       note="same line shape as scripts/verify_delivery_state_monotonicity.sh; "
            "send_prepared is the state=pending detail=message_prepared_local_history_written case "
            "(MeshRepository.kt:5877), delivered carries delivery_receipt_status (2705)"),
    _m("android_rx_message", "rx_decrypted",
       r"onMessageReceived pairing: sender=(?P<peer>\S+) .*?messageId=(?P<msg>\S+) kind=(?P<kind>\S+)",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2201",
       inferred=True,
       note="core hands the callback an already-decrypted message, so reaching it "
            "implies decrypt OK; no explicit decrypt line exists (gap G2)"),
    _m("android_rx_dup_stored", "rx_history",
       r"Duplicate inbound (?:message )?(?P<msg>\S+) (?:already stored|from (?P<peer>\S+))",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2114",
       inferred=True,
       note="only fires for a re-delivery of a message already in history "
            "(second site at :2455 is debug); first-time history write is silent (gap G3)"),
    _m("android_receipt_rx", "receipt_received",
       r"\[RECEIPT-RX\] Received from core: msg=(?P<msg>\S+) status=(?P<status>\S+)",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2522"),
    _m("android_receipt_history", "history_delivered",
       r"\[RECEIPT-RX\] History updated: msg=(?P<msg>\S+)",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2631"),
    _m("android_receipt_sent", "receipt_sent",
       r"Targeted delivery receipt sent for (?P<msg>\S+) to (?P<peer>\S+)",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2945",
       note="Timber.d: absent from release-level logs"),
    _m("android_delivery_attempt_acked", "transport_ack",
       r"delivery_attempt msg=(?P<msg>\S+) .*outcome=acked",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:12012",
       evidence=False, note="transport ACK; never scored"),
    _m("android_invite_imported", "invite_imported",
       r"Ledger: imported (?P<n>\d+) bootstrap seed\(s\) from join bundle",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:6603"),
    _m("android_seed_relay_dialed", "bootstrap_dial",
       r"Dialed seed relay from ledger: (?P<addr>\S+)",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:6009"),
    _m("android_peer_connection", "peer_identified",
       r"UNIFICATION peer_connection: peerId=(?P<peer>\S+) isKnownContact=\S+ agent=(?P<agent>\S+)",
       "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:1786",
       note="peerId is truncated to 16 chars; peers are matched by prefix"),

    # ---- message lifecycle: CLI / core (tracing) ---------------------------
    _m("cli_ack_sent", "receipt_sent",
       r"Sending delivery ACK for (?P<msg>\S+) to (?P<peer>\S+)",
       "cli/src/main.rs:3339 (and :4590)",
       extra_events=("rx_decrypted",), inferred=True,
       note="emitted only after core.receive_message() returned Ok, so it also proves "
            "decrypt (inferred); history write is not logged (gap G3)"),
    _m("cli_ack_received", "receipt_received",
       r"Delivery ACK received from (?P<peer>\S+): msg_id=(?P<msg>\S+)",
       "cli/src/main.rs:4608 (INFO) and :3455 (DEBUG, invisible at default level)"),
    _m("core_history_delivered", "history_delivered",
       r"Successfully marked message (?P<msg>\S+) as delivered",
       "core/src/store/history.rs:342"),
    _m("core_transport_ack", "transport_ack",
       r"\[OK\] Message delivered successfully to (?P<peer>\S+) \((?P<ms>\d+)ms\)",
       "core/src/transport/swarm.rs:5144",
       evidence=False, note="swarm-level request/response ACK; no message id; never scored"),
    _m("legacy_msg_rx", "rx_history",
       r"msg_rx_processed .*?msg=(?P<msg>\S+)",
       "NOT-IN-CODE (named in docs/RELAY_OPERATOR_GUIDE.md:327 and "
       "scripts/verify_receipt_convergence.sh; no emitter on origin/main)",
       extra_events=("rx_decrypted",),
       note="proposed shape: msg_rx_processed peer=<id> msg=<id> decrypt=ok history=written"),

    # ---- topology / identify ----------------------------------------------
    _m("core_identified_peer", "peer_identified",
       r"Identified peer (?P<peer>\S+) - agent: (?P<agent>[^,]+),",
       "core/src/transport/swarm.rs:6518"),
    _m("core_connection_established", "connection_established",
       r"Connection established to \[(?P<peer>[0-9a-f, ]+)\] via (?P<transport>\w+)",
       "core/src/transport/manager.rs:399",
       note="peer is the first 8 bytes of the Ed25519 key as hex (not a libp2p id)"),
    _m("cli_peer_disconnected", "peer_disconnected",
       r"Peer disconnected: (?P<peer>\S+)",
       "cli/src/main.rs:4428; core/src/mobile_bridge.rs:1557"),
    _m("core_bootstrap_dial", "bootstrap_dial",
       r"\[OK\]\s+Dialing bootstrap: (?P<addr>\S+)",
       "core/src/transport/swarm.rs:4256 (and :8704)"),
    _m("cli_seed_dial_connected", "seed_dial",
       r"\[SEED-DIAL\] sweep (?P<sweep>\d+): (?P<n>\d+) candidate\(s\), peers=0 -- (?P<outcome>.*)",
       "cli/src/seed_dial.rs:85 (connected) / :90 (dial outcome) / :76 (empty)"),
    _m("cli_seed_dial_watch", "seed_dial",
       r"\[SEED-DIAL\] peers=(?P<peers>\d+) -- connected",
       "cli/src/seed_dial.rs:69", note="steady-state heartbeat; DEBUG level"),
    _m("core_outbox_reconnect", "peer_reconnect",
       r"Peer identified; triggering outbox flush.*?peer_id[=:]\"?(?P<peer>[0-9A-Za-z]+)",
       "core/src/iron_core.rs:3337"),

    # ---- ledger exchange ---------------------------------------------------
    _m("ledger_started", "ledger_exchange_started",
       r"Started core ledger exchange with newly connected peer (?P<peer>\S+)",
       "core/src/transport/swarm.rs:7147 (and :9999 wasm)"),
    _m("ledger_rx_offer", "ledger_exchange_rx",
       r"Ledger exchange from (?P<peer>\S+): offered (?P<n>\d+) peer entries",
       "core/src/transport/swarm.rs:5811"),
    _m("ledger_reply", "ledger_exchange_reply",
       r"Ledger exchange reply to (?P<peer>\S+): sending (?P<n>\d+) peer entries",
       "core/src/transport/swarm.rs:5963", note="DEBUG level"),
    _m("ledger_response", "ledger_exchange_response",
       r"Ledger exchange response from (?P<peer>\S+): they learned (?P<learned>\d+) new peers, sent (?P<n>\d+) back",
       "core/src/transport/swarm.rs:5992"),
    _m("cli_ledger_learned", "ledger_learned_count",
       r"Learned (?P<n>\d+) new peers from (?P<peer>\S+)",
       "cli/src/main.rs:4440",
       note="count only: no peer id / address of what was learned (gap G1)"),
    _m("ledger_self_rejected", "ledger_self_entry_rejected",
       r"refusing a shared ledger row that claims our own identity",
       "core/src/store/ledger_entry.rs:2200"),
    _m("ledger_address_learned", "ledger_address_learned",
       r"ledger_address_learned peer=(?P<peer>\S+) via=(?P<via>\S+) addr=(?P<addr>\S+)",
       "core/src/store/ledger_entry.rs:merge_shared_entries_via (format: core/src/message_events.rs)",
       note="explicit proof a node learned <peer>'s address; via=ledger_exchange:<source peer16> "
            "from the CLI path, via=unknown from legacy callers"),

    # ---- relay custody -----------------------------------------------------
    _m("custody_audit_count", "custody_audit",
       r"Relay custody audit log count: (?P<n>\d+)",
       "core/src/transport/swarm.rs:4750"),
    _m("custody_accept", "relay_custody_accept",
       r"custody_accept msg=(?P<msg>\S+) from=(?P<peer>\S+) dest=(?P<dest>\S+)",
       "core/src/store/relay_custody.rs:900 (format: core/src/message_events.rs)",
       note="relay accepted custody of a message; msg id is sanitized and length-capped"),

    # ---- receiver legs (core, both CLI/AWS and Android via IronCore) -------
    _m("core_rx_decrypt", "rx_decrypted",
       r"rx_decrypt msg=(?P<msg>\S+) from=(?P<peer>\S+) type=(?P<kind>\S+) result=ok",
       "core/src/iron_core.rs:3910 (format: core/src/message_events.rs fmt_rx_decrypt)",
       note="explicit receiver decrypt leg"),
    _m("core_rx_history_ok", "rx_history",
       r"rx_history msg=(?P<msg>\S+) from=(?P<peer>\S+) result=ok dup=(?P<dup>\S+) hidden=(?P<hidden>\S+)",
       "core/src/iron_core.rs:4118 (format: core/src/message_events.rs fmt_rx_history)",
       note="durable history write succeeded (or message already stored, dup=true)"),
    _m("core_rx_history_failed", "rx_history_failed",
       r"rx_history msg=(?P<msg>\S+) from=(?P<peer>\S+) result=failed dup=(?P<dup>\S+) hidden=(?P<hidden>\S+)",
       "core/src/iron_core.rs:4118",
       note="history write FAILED: never counts as the history leg"),

    # ---- observability markers (this change + sibling agent PRs) -----------
    _m("routing_peer_seen", "routing_peer_seen",
       r"\[ROUTING\] peer_seen peer=(?P<peer>\S+) source=(?P<source>\S+)",
       "core/src/iron_core.rs:routing_peer_seen (format: core/src/message_events.rs fmt_routing_peer_seen)",
       evidence=False, note="routing engine was told a peer was seen on a transport; rate limited per peer"),
    _m("transport_status", "transport_status",
       r"\[TRANSPORT\] kind=(?P<tkind>\S+) state=(?P<state>\S+)(?: peers=(?P<peers>\d+))? detail=(?P<reason>\S+)",
       "cli/src/transport_status.rs; android/.../transport/TransportStatus.kt; "
       "core/src/message_events.rs fmt_transport_status",
       evidence=False,
       note="per-transport availability on start/change + connected-peer counts every 5 min"),
    _m("rx_drop", "rx_drop",
       r"\[RX-DROP\] (?:msg=(?P<msg>\S+) )?stage=(?P<stage>\S+)(?: reason=(?P<reason>\S+))?(?: kind=(?P<kind>\S+))?",
       "sibling agent PR (format: [RX-DROP] msg=<id> stage=<stage> reason=<r>)",
       evidence=False, note="receiver dropped an inbound message; explains a PARTIAL"),
    _m("rx_drop_suppressed", "rx_drop_suppressed",
       r"\[RX-DROP\] suppressed=(?P<count>\d+) stage=(?P<stage>\S+)",
       "sibling agent PR #512 (format: [RX-DROP] suppressed=<n> stage=<s>)",
       evidence=False, note="rate-limited rx-drop lines folded into a count"),
    _m("rx_stall", "rx_stall",
       r"\[RX-STALL\](?: (?P<fields>.*))?$",
       "sibling agent PR #512 ([RX-STALL] receive-path stall watchdog)",
       evidence=False, note="receive path stalled; fields kept as raw sanitized text"),
    _m("mesh_stop", "mesh_stop",
       r"\[MESH-STOP\] (?P<phase>requested|swarm_shutdown|rust_stop|foreground_removed|complete)"
       r"(?: (?P<result>ok|timeout))?(?: ms=(?P<ms>\d+))?",
       "sibling agent PR (requested|swarm_shutdown ok|timeout ms=|rust_stop ok|timeout ms=|foreground_removed|complete)",
       evidence=False, note="mesh stop sequence; a timeout phase means the stop was not clean"),
    _m("invite_imported_marker", "invite_imported",
       r"\[INVITE\] imported source=(?P<source>\S+)(?: (?P<fields>.*))?$",
       "android/.../ui/join/JoinMeshScreen.kt (join_bundle); android/.../data/MeshRepository.kt importSeedAddresses (seed_import)",
       note="Android invite redeemed: counts only. The CLI/FFI invite redeem path is #486/#501 and "
            "is not on main yet, so it has no marker"),
]

# kinds of identity-envelope traffic that are metadata, not user chat; the
# correlator ignores them when forming the message set (mirrors the CLI's
# is_identity_metadata rule at cli/src/main.rs:3321 and Android MeshRepository.kt:2194 isChatEvent).
NON_CHAT_KINDS_PREFIXES = ("identity", "history_sync", "config")

# delivery_state monotonicity: same rule as
# scripts/verify_delivery_state_monotonicity.sh (after delivered only
# delivered; after failed only failed).
TERMINAL_STATES = {"delivered": {"delivered"}, "failed": {"failed"}}

SEND_PREPARED_DETAIL = "message_prepared_local_history_written"  # MeshRepository.kt:5877


def match_line(text: str) -> List[Tuple[Marker, Dict[str, Optional[str]]]]:
    """Return every (marker, groupdict) that matches `text`."""
    out = []
    for mk in MARKERS:
        m = mk.pattern.search(text)
        if m:
            out.append((mk, m.groupdict()))
    return out
