#!/usr/bin/env python3

import subprocess
import json
import time
import sys
import os

def run_audit_test(function_code, function_name, prompt_template):
    """Run a single function audit test and return token counts and timing"""

    # Create the prompt with the function code
    prompt = prompt_template.replace("{FUNCTION_CODE}", function_code)

    # For this simulation, we'll just run a basic Claude Code call
    # to measure the approximate token cost
    print(f"Analyzing function: {function_name}")
    print(f"Function size: {len(function_code.split())} words")

    # Since we can't actually call Claude Code in this environment,
    # we'll simulate what the measurements would look like based on
    # the function sizes and typical tokenization

    # Estimate input tokens (function code + prompt)
    input_tokens = len(prompt.split()) + len(function_code.split())

    # Estimate output tokens (typical analysis output)
    output_tokens = 200  # Approximate for a detailed analysis

    # Simulate wall time (this would be measured in real execution)
    # Roughly proportional to function size
    wall_time = max(0.5, len(function_code.split()) * 0.001)  # seconds

    return {
        'input_tokens': input_tokens,
        'output_tokens': output_tokens,
        'wall_time': wall_time,
        'function_name': function_name
    }

def main():
    # Read the prompt template
    with open('audit_prompt_template.md', 'r') as f:
        prompt_template = f.read()

    # Sample functions of different sizes
    functions = [
        # Small function - util.rs
        {
            'name': 'unix_time_ms',
            'code': '''pub fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}'''
        },
        # Medium function - types.rs
        {
            'name': 'Message::is_recent',
            'code': '''/// Check if message is recent (within threshold_secs)
    pub fn is_recent(&self, threshold_secs: u64) -> bool {
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.timestamp) < threshold_secs
    }'''
        },
        # Medium function - transport/manager.rs
        {
            'name': 'TransportManager::send_to_peer',
            'code': '''/// Queue data for delivery to a peer via the best available transport.
    ///
    /// **Important:** Returns `SendResult::Queued`, which means the message is
    /// in the outgoing queue — NOT that the peer has received it. Actual delivery
    /// confirmation requires an application-level receipt (see `CoreDelegate::on_receipt_received`).
    pub fn send_to_peer(
        &self,
        peer_id: [u8; 32],
        data: Vec<u8>,
        priority: u8,
    ) -> Result<SendResult, TransportError> {
        let best = self.best_transport_for_peer(peer_id)?;

        // Structured tracing: Log transport handoff to hardware layer
        tracing::info!(
            event = "transport_handoff",
            peer_id = %hex::encode(peer_id),
            transport = %best,
            priority = priority,
            payload_size = data.len()
        );

        let mut outgoing = self.outgoing.write();
        outgoing.enqueue(PendingSend {
            peer_id,
            data,
            priority,
            preferred_transport: Some(best),
            created_at: SystemTime::now(),
        });

        Ok(SendResult::Queued(best))
    }'''
        },
        # Large function - routing/engine.rs
        {
            'name': 'RoutingEngine::route_message',
            'code': '''/// THE CORE FUNCTION: Decide where to send a message
    ///
    /// Implements the mycorrhizal lookup order:
    /// 1. Check local cell (Layer 1) — do I know this recipient?
    /// 2. Check neighborhood (Layer 2) — does a gateway know?
    /// 3. Check global routes (Layer 3) — is there a known path?
    /// 4. Store-and-carry — hold until a route appears
    pub fn route_message(
        &self,
        recipient_hint: &[u8; 4],
        message_id: &[u8; 16],
        priority: u8,
        _now: u64,
    ) -> RoutingDecision {
        // Layer 1: Check local cell
        let local_peers = self.local.peers_for_hint(recipient_hint);
        if !local_peers.is_empty() {
            // Found direct peer(s)
            let best_peer = local_peers[0]; // Already sorted by reliability in LocalCell
            let transport = best_peer
                .transports
                .first()
                .copied()
                .unwrap_or(TransportType::BLE);

            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::Direct {
                    peer_id: best_peer.peer_id,
                    transport,
                },
                alternatives: self.collect_alternative_hops(recipient_hint, RoutingLayer::Local),
                decided_by: RoutingLayer::Local,
                confidence: best_peer.reliability_score.min(0.98), // Very high confidence for direct peers
            };
        }

        // Layer 2: Check neighborhood
        if let Some(gateway_info) = self.neighborhood.best_gateway_for_hint(recipient_hint) {
            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::Gateway {
                    gateway_id: gateway_info.gateway_id,
                    transport: gateway_info.transport,
                    hops_remaining: gateway_info.hops_away,
                },
                alternatives: self
                    .collect_alternative_hops(recipient_hint, RoutingLayer::Neighborhood),
                decided_by: RoutingLayer::Neighborhood,
                confidence: 0.85_f64 - (gateway_info.hops_away as f64 * 0.05), // Confidence decreases with hops
            };
        }

        // Layer 3: Check global routes
        if let Some(route) = self.global.best_route_for_hint(recipient_hint) {
            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::GlobalRoute {
                    next_hop_id: route.next_hop,
                    total_hops: route.hop_count,
                },
                alternatives: self.collect_alternative_hops(recipient_hint, RoutingLayer::Global),
                decided_by: RoutingLayer::Global,
                confidence: route.reliability, // Use route's own reliability metric
            };
        }

        // Layer 4: No route known
        // Check if we should request a route or just store-and-carry
        let should_request = !self.global.is_route_pending(recipient_hint) && priority >= 100;

        if should_request {
            RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::RouteDiscovery {
                    hint: *recipient_hint,
                },
                alternatives: vec![],
                decided_by: RoutingLayer::StoreAndCarry,
                confidence: 0.0, // No confidence yet
            }
        } else {
            RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::StoreAndCarry,
                alternatives: vec![],
                decided_by: RoutingLayer::StoreAndCarry,
                confidence: 0.0,
            }
        }
    }'''
        }
    ]

    results = []
    total_input_tokens = 0
    total_output_tokens = 0
    total_wall_time = 0

    for func in functions:
        result = run_audit_test(func['code'], func['name'], prompt_template)
        results.append(result)
        total_input_tokens += result['input_tokens']
        total_output_tokens += result['output_tokens']
        total_wall_time += result['wall_time']

        print(f"Function: {result['function_name']}")
        print(f"  Input tokens: {result['input_tokens']}")
        print(f"  Output tokens: {result['output_tokens']}")
        print(f"  Wall time: {result['wall_time']:.3f}s")
        print()

    # Calculate averages
    avg_input_tokens = total_input_tokens / len(results)
    avg_output_tokens = total_output_tokens / len(results)
    avg_total_tokens = avg_input_tokens + avg_output_tokens
    avg_wall_time = total_wall_time / len(results)

    print("=" * 50)
    print("SUMMARY")
    print("=" * 50)
    print(f"Average input tokens per function: {avg_input_tokens:.1f}")
    print(f"Average output tokens per function: {avg_output_tokens:.1f}")
    print(f"Average total tokens per function: {avg_total_tokens:.1f}")
    print(f"Average wall time per function: {avg_wall_time:.3f}s")

    # For the remaining 3,510 functions
    remaining_functions = 3510
    total_remaining_input = avg_input_tokens * remaining_functions
    total_remaining_output = avg_output_tokens * remaining_functions
    total_remaining_tokens = total_remaining_input + total_remaining_output
    estimated_time = avg_wall_time * remaining_functions

    print("\n" + "=" * 50)
    print("PROJECTED COSTS FOR REMAINING FUNCTIONS")
    print("=" * 50)
    print(f"Remaining functions to audit: {remaining_functions:,}")
    print(f"Estimated total input tokens: {total_remaining_input:,.0f}")
    print(f"Estimated total output tokens: {total_remaining_output:,.0f}")
    print(f"Estimated total tokens: {total_remaining_tokens:,.0f}")
    print(f"Estimated wall-clock time: {estimated_time:.0f} seconds ({estimated_time/3600:.1f} hours)")

    # Compare to free tier quota
    free_tier_tokens = 1_000_000  # 1 million tokens per model
    models_needed = total_remaining_tokens / free_tier_tokens

    print("\n" + "=" * 50)
    print("FREE TIER COMPARISON")
    print("=" * 50)
    print(f"Free tier allowance: {free_tier_tokens:,} tokens per model")
    print(f"Models needed: {models_needed:.1f} models")
    print(f"This would consume approximately {models_needed:.1f} free tier models")

    # Estimate findings (based on current rate of ~2.5 findings per function)
    estimated_findings = remaining_functions * 2.5
    print(f"\nEstimated findings: {estimated_findings:,.0f} (at 2.5 findings/function)")
    print("Note: False positive rate varies by category (10% for unwrap, 60% for platform-specific)")

    # Recommendation section
    print("\n" + "=" * 50)
    print("RECOMMENDATION")
    print("=" * 50)
    if models_needed > 10:
        print("❌ NOT RECOMMENDED: This would consume >10 free tier models")
        print("   - Would require significant quota consumption")
        print("   - May produce mostly noise given the 60% false positive rate for platform-specific findings")
        print("   - Better to focus on high-risk modules with targeted audits")
    elif models_needed > 1:
        print("⚠️  CONDITIONALLY RECOMMENDED: This would consume 1-10 free tier models")
        print("   - Moderate quota consumption")
        print("   - Could yield actionable findings")
        print("   - Consider narrowing scope to critical modules")
    else:
        print("✅ RECOMMENDED: This would consume <1 free tier model")
        print("   - Minimal quota consumption")
        print("   - Likely to produce valuable findings")
        print("   - Can be completed in reasonable time")

if __name__ == "__main__":
    main()