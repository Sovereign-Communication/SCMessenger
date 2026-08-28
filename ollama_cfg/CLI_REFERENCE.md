# SCMessenger CLI Reference Guide
> Consumed by `scm-expert:latest` (llama3.2:3b). Keep this document concise —
> the model reads it via `raw` when primary commands are insufficient.

---

## 1. Identity Management
| Action           | Raw Subcommand                                              |
|------------------|-------------------------------------------------------------|
| Initialize       | `raw init [--name <nickname>]`                              |
| Show identity    | `raw identity show`                                         |
| Set nickname     | `raw identity set-name <name>`                              |
| Device ID        | `raw identity device-id`                                    |
| Export backup    | `raw identity export --passphrase <secret> [--output <file>]` |
| Import backup    | `raw identity import --passphrase <secret> [--backup <payload> \| --input <file>]` |
| Sign data        | `raw identity sign-data <hex_data>`                         |
| Verify signature | `raw identity verify-signature --data-hex <hex> --signature-hex <hex> --public-key-hex <hex>` |

---

## 2. Contact Management
| Action          | Raw Subcommand                                              |
|-----------------|-------------------------------------------------------------|
| Add contact     | `raw contact add <peer_id> <public_key> [--name <nickname>]` |
| List contacts   | `contact list` *(primary command)*                          |
| Remove contact  | `raw contact remove <contact_name_or_id>`                   |
| Search          | `raw contact search <query>`                                |
| Set nickname    | `raw contact set-nickname <contact> <nickname>`             |

> **Note:** PeerId starts with `12D3Koo`. Public Key is 64 hex chars.

---

## 3. Node Control
| Action         | Raw Subcommand                                              |
|----------------|-------------------------------------------------------------|
| Start node     | `start` *(primary command)*                                 |
| Stop node      | `stop` *(primary command)*                                  |
| Status         | `status` *(primary command)*                                |
| Headless relay | `raw relay [--listen <multiaddr>] [--http-port <port>] [--name <name>]` |

---

## 4. Messaging & History
| Action              | Raw Subcommand                                              |
|---------------------|-------------------------------------------------------------|
| Send message        | `send <peer_id> "msg"` *(primary command)*                  |
| View history        | `raw history [--peer <peer_id>] [--limit <n>] [--search <query>]` |
| Clear all history   | `raw history-clear --yes`                                   |
| Prune before time   | `raw history-prune-before <unix_timestamp>`                 |
| Delete conversation | `raw history-clear-conversation <peer_id>`                  |

---

## 5. Network & Security
| Action       | Raw Subcommand                                              |
|--------------|-------------------------------------------------------------|
| Swarm stats  | `raw swarm stats`                                           |
| Block peer   | `raw block add <peer_id> [--reason <text>]`                 |
| Unblock peer | `raw block remove <peer_id>`                                |

---

## 6. Discovery
> **Status:** mDNS/LAN and BLE discovery auto-start with the node (background).
> Manual controls are available via `discovery status|peers`.

| Action           | Command                                                      |
|------------------|--------------------------------------------------------------|
| Discovery status | `python scripts/core_cli_driver.py discovery status`         |
| List live peers  | `python scripts/core_cli_driver.py discovery peers`          |
| Force mDNS scan  | `raw discovery scan` *(if available on build)*               |

> **WiFi-Aware:** Android-only. No CLI equivalent planned yet.

---

## 7. Configuration
| Action      | Raw Subcommand              |
|-------------|-----------------------------|
| Set value   | `raw config set <key> <value>` |
| Get value   | `raw config get <key>`      |
| List all    | `raw config list`           |

---

## 8. ISSUE Schema (Orchestrator Protocol)
When the model detects an error or anomaly, it emits a structured `ISSUE` block.
The orchestrator parses this to dispatch a fix agent.

```json
ISSUE {
  "type": "crash|config|network|identity|timeout|unknown",
  "severity": "low|medium|high|critical",
  "component": "daemon|transport|identity|contact|mesh",
  "summary": "one-sentence human description",
  "evidence": "relevant log line, JSON field, or error message",
  "suggested_action": "restart|rebuild|diagnose|escalate"
}
```

### Severity Guide
| Severity | When to use                                              |
|----------|----------------------------------------------------------|
| low      | Warning only, node still functional                      |
| medium   | Feature degraded, workaround exists                      |
| high     | Core function broken, no workaround                      |
| critical | Node crashed or data at risk, immediate action needed    |

### Suggested Actions
- `restart` — Daemon needs a stop/start cycle
- `rebuild` — Binary is stale; run `cargo build -p scmessenger-cli`
- `diagnose` — Run `daemon-log 50` and inspect; launch a debug agent
- `escalate` — Requires human or cloud-model intervention

---

## 9. Driver Usage
```
python scripts/core_cli_driver.py <primary_command> [args]
python scripts/core_cli_driver.py raw <scm_subcommand_without_prefix> [args]
```

All output is JSON with a `"status"` field. On error, check `"reason"` and `"stderr"`.
