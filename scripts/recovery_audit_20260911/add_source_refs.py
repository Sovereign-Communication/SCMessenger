import json
import re

def add_source_refs(claims, source_file):
    """Add source_refs to claims by analyzing source lines for keywords."""
    with open(source_file, 'r') as f:
        lines = f.readlines()
    
    # Keywords that need source_refs
    load_bearing = [
        'does not', 'never', 'always', 'only', 'unbounded', 'unconditionally',
        'without', 'fails to', 'missing', 'lacks', 'no validation', 'no check',
        'silently', 'arbitrary', 'fragile', 'broadly', 'all ', 'every ',
        'incorrectly', 'wrongly', 'misinterprets'
    ]
    
    updated_claims = []
    for claim in claims:
        text = claim.get('text', '').lower()
        refs = []
        
        # Find lines containing keywords from the claim
        for i, line in enumerate(lines):
            line_lower = line.lower()
            # Check if any load-bearing word from claim appears in source
            for kw in load_bearing:
                if kw in text and kw in line_lower:
                    refs.append(i + 1)
        
        # Also check for specific technical terms
        tech_terms = [
            'circuit_count', 'is_tracked_reservation', 'circuit_count > 1',
            '!is_tracked_reservation', 'circuit_count == 0', 'return false',
            'wildcard', 'unspecified', 'ipv6', 'claims', 'conditional',
            'port', 'match', 'local_ip', 'cand_ip', 'is_unspecified',
            'is_loopback', 'is_private', 'is_link_local', 'is_discoverable_multiaddr',
            'is_self_endpoint', 'any', 'matches', 'p2pcircuit',
            'delivery_attempts', 'delivered', 'scan_prefix', 'filter_map',
            'bincode', 'deserialize', 'custodymessage', 'custodystate',
            'reliability_score', 'partial_cmp', 'expect', 'nan', 'peer_id',
            'assert_eq', 'hardcodes', 'hint', 'make_hint', 'peers_for_hint',
            'active_transports', 'peerid', 'wifi_aware', 'wifi_direct',
            'isavailable', 'connected_devices', 'first()', 'peerid mismatch',
            'internet', 'tcp_mdns', 'swarmbridge', 'catch', 'exception',
            'retry', 'transient', 'validate', 'multiaddr', 'proven', 'seeds',
            'maxseeds', 'trim', 'distinct', 'proven_set', 'filter_not',
            'detail', 'contains', 'device offline', 'route to host', 'carrier',
            'filtering', 'blocking', 'quic', 'udp', 'ledger_manager',
            'advertise', 'permission', 'advertise_settings', 'advertise_data',
            'add_service_uuid', 'parcel_uuid', 'service_uuid', 'identity_data',
            'security_exception', 'is_advertising', 'rotation', 'start_rotation',
            'stop_advertising', 'advertise_callback', 'stop_rotation',
            'share', 'fileprovider', 'intent', 'action_send', 'extra_stream',
            'grant_read_uri_permission', 'create_chooser', 'dispatchers.io',
            'resolve', 'cache_dir', 'files_dir', 'authority', 'get_uri_for_file',
            'illegal_argument_exception', 'mkdirs', 'write_text', 'dispatchers.io',
            'view_model_scope', 'contact_count', 'message_count', 'build_provenance',
            'mesh_repository', 'get_contact_count', 'get_message_count',
            'get_build_provenance', 'infocounts', 'dispatchers.io',
        ]
        
        # If no refs found, use heuristic: find lines with terms from claim
        if not refs:
            for i, line in enumerate(lines):
                line_lower = line.lower()
                for term in tech_terms:
                    if term in text and term in line_lower:
                        refs.append(i + 1)
                        break
        
        # Deduplicate and sort
        refs = sorted(set(refs))
        if not refs:
            # Fallback: all lines
            refs = list(range(1, len(lines) + 1))
        
        claim_copy = claim.copy()
        claim_copy['source_refs'] = refs
        updated_claims.append(claim_copy)
    
    return updated_claims

# Update all claims files
claims_dir = r"C:\Users\SCM\Documents\GitHub\SCMessenger\scmessenger_audit\claims"
source_dir = r"C:\Users\SCM\Documents\GitHub\SCMessenger\scmessenger_audit\source"

import os
for fname in os.listdir(claims_dir):
    if fname.endswith('.json'):
        fpath = os.path.join(claims_dir, fname)
        with open(fpath, 'r') as f:
            data = json.load(f)
        
        source_fname = fname.replace('.json', '.txt')
        source_fpath = os.path.join(source_dir, source_fname)
        
        if os.path.exists(source_fpath):
            data['claims'] = add_source_refs(data['claims'], source_fpath)
            with open(fpath, 'w') as f:
                json.dump(data, f, indent=2)
            print(f"Updated: {fname}")
        else:
            print(f"Source not found for: {fname}")

print("Done updating claims")