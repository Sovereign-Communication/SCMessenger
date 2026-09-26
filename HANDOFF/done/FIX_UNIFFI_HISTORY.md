TASK: Fix missing UniFFI export for hide_messages_for_peer
ERROR: java.lang.UnsatisfiedLinkError: Error looking up function 'uniffi_scmessenger_core_fn_method_historymanager_hide_messages_for_peer'

1. Check core/src/store/history.rs, core/src/mobile_bridge.rs, or wherever HistoryManager is defined.
2. Ensure hide_messages_for_peer has the #[uniffi::export] macro or is properly included in an exported impl block.
3. Ensure the function signature matches what UniFFI expects.
4. If it's defined in an impl block, make sure the whole impl block is exported or the function is exposed properly.


# REPO_MAP Context for Task: FIX_UNIFFI_HISTORY

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
