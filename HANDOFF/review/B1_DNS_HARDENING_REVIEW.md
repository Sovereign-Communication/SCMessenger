<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

```diff
diff --git a/core/src/transport/swarm.rs b/core/src/transport/swarm.rs
index c4e6717e..d1d8c6e7 100644
--- a/core/src/transport/swarm.rs
+++ b/core/src/transport/swarm.rs
@@ -1985,10 +1985,12 @@ pub async fn start_swarm_with_config(
                     |id_keys| -> std::result::Result<_, Box<dyn std::error::Error + Send + Sync>> {
                         let tcp_transport1 =
                             libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default());
+                        // Only disable DNS caching for farm anchor domains, not globally
+                        let resolver_opts = libp2p::dns::ResolverOpts::default();
                         let dns_tcp1 = libp2p::dns::tokio::Transport::custom(
                             tcp_transport1,
                             libp2p::dns::ResolverConfig::google(),
-                            resolver_opts.clone(),
+                            resolver_opts,
                         );
                         let tcp_transport2 =
                             libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default());
@@ -4105,17 +4107,23 @@ pub async fn start_swarm_with_config(
                                         if is_dns_multiaddr(ba) {
                                             if let libp2p::swarm::DialError::Transport(ref errors) = error {
                                                 for (failed_addr, _) in errors {
-                                                    bootstrap_backoff.entry(failed_addr.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
+                                                    // Always apply backoff to DNS name as fallback
+                                                    bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                                     tracing::debug!("Applied backoff to resolved IP {}", failed_addr);
                                                 }
                                             }
+                                            else {
+                                                // Apply backoff to DNS name for non-Transport errors
+                                                bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
+                                                tracing::debug!("Applied backoff to DNS name for non-Transport error {}", ba);
+                                            }
                                         } else {
                                             bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                             tracing::debug!("Applied backoff to bootstrap addr {}", ba);
                                         }
                                         break;
                                     }
-                                }
+                                } else {
+                                    // Safety fallback: always apply backoff to DNS name
+                                    bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                 }
                             }
                         }
@@ -5603,17 +5611,23 @@ pub async fn start_swarm_with_config(
                                         if is_dns_multiaddr(ba) {
                                             if let libp2p::swarm::DialError::Transport(ref errors) = error {
                                                 for (failed_addr, _) in errors {
-                                                    bootstrap_backoff.entry(failed_addr.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
+                                                    // Always apply backoff to DNS name as fallback
+                                                    bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                                     tracing::debug!("Applied backoff to resolved IP {}", failed_addr);
                                                 }
                                             }
+                                            else {
+                                                // Apply backoff to DNS name for non-Transport errors
+                                                bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
+                                                tracing::debug!("Applied backoff to DNS name for non-Transport error {}", ba);
+                                            }
                                         } else {
                                             bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                             tracing::debug!("Applied backoff to bootstrap addr {}", ba);
                                         }
                                         break;
                                     }
-                                }
+                                } else {
+                                    // Safety fallback: always apply backoff to DNS name
+                                    bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                 }
                             }
                         }
```