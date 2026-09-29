import re

# UDL-declared methods for IronCore (including constructors)
udl_methods = {
    "new", "with_storage", "with_storage_and_logs",
    "start", "stop", "is_running", "grant_consent",
    "initialize_identity", "get_identity_info", "set_nickname",
    "get_device_id", "get_seniority_timestamp", "get_registration_state",
    "export_identity_backup", "import_identity_backup", "sign_data",
    "verify_signature", "extract_public_key_from_peer_id",
    "resolve_identity", "resolve_to_identity_id",
    "prepare_message", "prepare_message_with_id", "prepare_receipt",
    "prepare_cover_traffic", "outbox_count", "inbox_count",
    "mark_message_sent", "classify_notification", "block_peer",
    "unblock_peer", "is_peer_blocked", "list_blocked_peers",
    "blocked_count", "block_and_delete_peer", "set_delegate",
    "update_disk_stats", "perform_maintenance", "record_log",
    "export_logs", "get_audit_log", "get_audit_events_since",
    "export_audit_log", "validate_audit_chain", "drift_activate",
    "drift_deactivate", "drift_network_state", "drift_store_size",
    "record_abuse_signal", "get_peer_reputation", "peer_rate_limit_multiplier",
    "get_privacy_config", "set_privacy_config", "relay_jitter_delay",
}

with open("core/src/iron_core.rs", "r", encoding="utf-8") as f:
    lines = f.readlines()

in_exported_impl = False
brace_depth = 0
out_lines = []

for i, line in enumerate(lines):
    stripped = line.strip()
    
    # Detect the exported impl block
    if stripped == "#[uniffi::export]" and i + 1 < len(lines) and lines[i + 1].strip().startswith("impl IronCore {"):
        in_exported_impl = True
        brace_depth = 0
        out_lines.append(line)
        continue
    
    if in_exported_impl:
        # Track braces
        brace_depth += line.count("{") - line.count("}")
        
        # Check if this line declares a public method
        match = re.match(r'^(\s+)pub fn (\w+)\(', line)
        if match and brace_depth > 0:
            indent = match.group(1)
            method_name = match.group(2)
            if method_name not in udl_methods:
                line = line.replace(f"{indent}pub fn ", f"{indent}pub(crate) fn ")
        
        if brace_depth == 0 and stripped == "}":
            in_exported_impl = False
    
    out_lines.append(line)

with open("core/src/iron_core.rs", "w", encoding="utf-8") as f:
    f.writelines(out_lines)

print("Done")
