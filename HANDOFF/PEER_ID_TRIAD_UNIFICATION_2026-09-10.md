# PeerIdTriad — PeerID / Public Key / Identity Hash unification

Status: landed on `unified/v040-3node-parity` @ `8c19f900`
Date: 2026-09-10

## The three identifiers

| Name | Shape | Example (Windows) | Use |
|---|---|---|---|
| **libp2p PeerID** | base58 `12D3KooW…` | `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` | Transport / dial / multiaddr |
| **public_key** | 64-hex Ed25519 (`pk:`) | `30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e` | Crypto, contact canonical id |
| **identity_id** | 64-hex blake3 of pubkey (`id:`) | `985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826` | Identity envelope / custody / history receipts |

Derivation (self-certifying, Ed25519):

```
public_key_hex  ⇄  libp2p_peer_id     (reversible identity multihash)
public_key_hex  →  identity_id        (blake3, one-way)
identity_id     →  public_key_hex     IMPOSSIBLE
```

## Canonical API

- `scmessenger_core::identity::PeerIdTriad::resolve(input)` accepts any flavor
  (`12D3…`, 64-hex pubkey, 64-hex identity_id, optional `pk:`/`id:` prefixes)
  and returns the full triad + `self_certifying` flag.
- **HTTP** (live node):
  - `GET /api/identity` → includes `triad` object
  - `GET /api/peers` → each peer includes `triad`
  - `GET /api/peer-resolve?input=<any id>` → resolve without a peer connection

## Logging contract

| Surface | Format |
|---|---|
| Windows/AWS | `[IDENTITY-TRIAD] p2p=… pk:… id:…` (startup) |
| Android | `SC_IDENTITY_OWN p2p_id=… pk=… id=…` |
| Contact writes | `contacts_canonical_hex_live from=12D3… to=<pk>` |

## Fleet registry (candidate 238a8c53 / 8c19f900)

Re-derive after any cutover via `/api/peer-resolve` or `/api/identity`.

| Node | libp2p PeerID | public_key | identity_id |
|---|---|---|---|
| Windows CLI | `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` | `30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e` | `985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826` |
| AWS relay | `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31` | `69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c` | `37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006` |
| Pixel 6a | `12D3KooWR9ioPPRJ2tGPbWj9NVKXAve2iwZX4csLbDd1Tn6Hpi3B` | `e3d4aaecf5b02fc7112a0d17b90d1dbe319a9ef7eef93bc7f099821e935ec7fa` | `9a23057410a35584e79747fb0357821bf07bb6bd9a757d1c3e827edea3faab36` |

Live verification 2026-09-10 on Windows node `8c19f900`:
- All three flavors of Windows self-identity resolve to one triad
  (`self_certifying=true` for peer_id and pubkey; identity_id correctly
  does not invert to a pubkey).
- `/api/peers` returns full triads for Phone + AWS (`self_cert=true`).
- Phone mesh: `peersDiscovered=2`, identity `9a230574…` cached.

## App rules (do not confuse again)

1. **Dial / multiaddr / swarm peers** → always libp2p PeerID.
2. **Contacts, encryption, receipts, history canonical key** → always public_key.
3. **identity_id** is metadata + custody; never use it as a dial address.
4. When an id arrives that is 64-hex: if it is a valid Ed25519 point treat as
   pubkey; else treat as identity_id (cannot invert). Dual-match history
   queries already do this (`history_peer_matches`).

## Verification

```powershell
# any flavor → full triad
curl "http://127.0.0.1:9876/api/peer-resolve?input=12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw"
curl "http://127.0.0.1:9876/api/peer-resolve?input=30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e"
```

Unit tests: `cargo test -p scmessenger-core --lib -- peer_id_triad`
