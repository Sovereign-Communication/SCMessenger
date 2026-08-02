# SCMessenger: Private 1:1 Messaging (Not Released Yet)

*Still buggy. But thanks to Freenet, one step closer.*

---

## What Is SCMessenger?

A peer-to-peer messenger for **direct 1:1 only** — no groups, no servers, no phone numbers.

- **Identity** = your Ed25519 public key (64 hex chars)
- **Transport** = libp2p (TCP/QUIC/WS/WebRTC)
- **Encryption** = double ratchet + post-quantum hybrid
- **Sync** = Drift Protocol (compact frames, IBLT reconciliation)
- **No blockchain, no tokens, no consensus**

You install, share your key, message. That's the goal. We're not there yet.

---

## What We're Taking From Freenet (Three Things)

I read their whitepaper and source. Three concrete things we're adopting:

| Thing | Why |
|-------|-----|
| **NAT traversal: 3s deadline, 200ms cadence, 40 attempts, dual-sided punch** | Our `nat.rs` had the framework but the hole-punch was effectively a stub. Freenet's works in production. |
| **Rate-limited asymmetric decryption (1s min interval)** | Prevents CPU exhaustion from junk intro packets. Simple, effective. |
| **Summary/delta sync interface** | Cleaner abstraction for "only fetch what's missing." Refactoring Drift to match. |

That's it. Not their routing, not WASM contracts, not group chat primitives. Just the transport plumbing that works.

---

## What's Next

- **Week 1-2**: Replace NAT stub with Freenet's proven approach
- **Week 2-3**: Add decryption rate limiting
- **Week 3-4**: Run UDP hole-punch in parallel with libp2p `dcutr` (first wins)
- **Ongoing**: Simulation tests in CI (Turmoil — their idea, solid)

We'll upstream any generic transport fixes.

---

## Want to Help?

SCMessenger is early. Rust core (`core/`), Android (`android/`), iOS (`iOS/`). 

**Needed:** mobile NAT testing, UI polish, protocol review, CI hardening.

`github.com/your-org/SCMessenger` — issues tagged `good first issue` are real.

---

## Thanks, Freenet

You built transport plumbing that works and documented it honestly. That's rare. We're using it.

*No official affiliation. Just inspired.*