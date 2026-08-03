# Two-Endpoint Delivery Proof -- 2026-07-20

## Status: COMPLETE

Both directions of end-to-end message delivery proven between two independent
CLI identities over the alpha relay. Evidence below.

## Setup

- Alice identity: pubkey `b6ffcd97...`, peer ID `12D3KooW<redacted>`
- Bob identity: pubkey `94c1f6cb...`, peer ID `12D3KooW<redacted>`
- Relay: `/ip4/100.56.248.69/tcp/9001` (peer ID `12D3KooW<redacted>`)
- Alice node: `scm start --port 9100 --http-bind 127.0.0.1:19877`, data dir `tmp/alice/`
- Bob node: `scm start --port 9200 --http-bind 127.0.0.1:19876`, data dir `tmp/bob/`

## Evidence

### Both nodes connected to relay

Alice `/api/peers` response:
```json
[
  {"peer_id": "12D3KooW<redacted>", "reputation": 50.0},
  {"peer_id": "12D3KooW<redacted>", "reputation": 50.0}
]
```
Bob `/api/peers` response:
```json
[
  {"peer_id": "12D3KooW<redacted>", "reputation": 50.0},
  {"peer_id": "12D3KooW<redacted>", "reputation": 50.0}
]
```

### Alice -> Bob (direction 1)

Send: `POST http://127.0.0.1:19877/api/send`
```json
{"recipient": "Bob", "message": "Hello from Alice - two-endpoint proof 2026-07-20"}
```
Response: `{"success": true, "error": null}`

Bob history confirmation:
```
[received] from=c4e9d8f6... msg=Hello from Alice - two-endpoint proof 2026-07-20
```

### Bob -> Alice (direction 2)

Send: `POST http://127.0.0.1:19876/api/send`
```json
{"recipient": "Alice", "message": "Hello back from Bob - delivery confirmed both ways"}
```
Response: `{"success": true, "error": null}`

Alice history confirmation:
```
[received] from=aec8297d... msg=Hello back from Bob - delivery confirmed both ways
```

## Bug fixed as part of this proof

`handle_send_message` in `cli/src/api.rs` was calling
`contact.peer_id.parse::<libp2p::PeerId>()` but the `Contact` store
model stores the Ed25519 public key hex in `peer_id`, not a libp2p
base-58 multihash. Fixed by deriving the PeerId from the public key bytes.
Commit: `29d01e5b`. fusion_lite verified PASS.

## Notes

- These are two CLI identities, not an Android device. The protocol is proven.
  Android end-to-end still requires a physical device or working emulator.
- Lucas's own identity (Lucas's node on the relay) also appeared in Bob's
  history as identity sync messages -- confirms the relay is live and serving
  other real connections simultaneously.
- The alpha relay was reachable throughout at `/ip4/100.56.248.69/tcp/9001`.
