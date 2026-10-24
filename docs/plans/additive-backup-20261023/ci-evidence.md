# CI evidence and limits

Status: Proposed planning evidence; no backup execution
Last updated: 2026-10-24 UTC

Captured: 2026-10-24T09:00:15.905985+00:00. Configuration read only; no setting, workflow, hook or run changed. Repo hook arrays are complete paginated responses. Installed Apps, org hooks and external CI are NOT proven absent. Backup execution must obtain owner attestation/audit and disabled Actions read-back. Planning branch is non-main, has no PR/tag/dispatch and uses [skip ci].

```json
{
  "repositories": {
    "Sovereign-Communication/SCMessenger": {
      "permissions": {
        "enabled": true,
        "allowed_actions": "all",
        "sha_pinning_required": false
      },
      "repo_hooks": [
        []
      ]
    },
    "Treystu/SCMessenger-backup": {
      "permissions": {
        "enabled": true,
        "allowed_actions": "all",
        "sha_pinning_required": false
      },
      "repo_hooks": [
        []
      ]
    }
  },
  "canonical_main": "941e669da7b1fe9c0c5350dcd010e0ee5c6ecd58",
  "workflow_count": 20,
  "workflow_events": [
    {
      "path": ".github/workflows/auto-label.yml",
      "on": {
        "pull_request": {
          "types": [
            "opened",
            "synchronize",
            "reopened"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/auto-tag-release.yml",
      "on": {
        "workflow_dispatch": {
          "inputs": {
            "dispatch_release": {
              "description": "Start the Multi-Platform Release Pipeline for the new tag",
              "required": "false",
              "type": "boolean",
              "default": "true"
            }
          }
        }
      }
    },
    {
      "path": ".github/workflows/ci.yml",
      "on": {
        "push": {
          "branches": [
            "main"
          ]
        },
        "pull_request": {
          "branches": [
            "main"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/cross-platform-test.yml",
      "on": {
        "workflow_dispatch": {
          "inputs": {
            "sprint_id": {
              "description": "Sprint identifier for tracking (e.g., sprint-42)",
              "required": "true",
              "type": "string"
            },
            "git_branch": {
              "description": "Branch to build and test",
              "required": "true",
              "default": "main",
              "type": "string"
            }
          }
        }
      }
    },
    {
      "path": ".github/workflows/cross.yml",
      "on": {
        "push": {
          "branches": [
            "main"
          ],
          "paths-ignore": [
            "**.md",
            "HANDOFF/**",
            "docs/**"
          ]
        },
        "pull_request": {
          "branches": [
            "main"
          ],
          "paths-ignore": [
            "**.md",
            "HANDOFF/**",
            "docs/**"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/desktop.yml",
      "on": {
        "push": {
          "branches": [
            "main"
          ],
          "paths": [
            "shared/**",
            "desktop_bridge/**",
            "scripts/build_desktop.*"
          ]
        },
        "pull_request": {
          "branches": [
            "main"
          ],
          "paths": [
            "shared/**",
            "desktop_bridge/**",
            "scripts/build_desktop.*"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/dist.yml",
      "on": {
        "workflow_dispatch": "",
        "push": {
          "tags": [
            "v*"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/docker-publish.yml",
      "on": {
        "push": {
          "branches": [
            "main"
          ],
          "paths-ignore": [
            "**.md",
            "HANDOFF/**",
            "docs/**"
          ]
        },
        "workflow_dispatch": ""
      }
    },
    {
      "path": ".github/workflows/docker-test-suite.yml",
      "on": {
        "push": {
          "branches": [
            "main"
          ],
          "paths-ignore": [
            "**.md",
            "HANDOFF/**",
            "docs/**"
          ]
        },
        "schedule": [
          {
            "cron": "17 6 * * 1"
          }
        ],
        "workflow_dispatch": ""
      }
    },
    {
      "path": ".github/workflows/hygiene.yml",
      "on": {
        "pull_request": {
          "branches": [
            "main"
          ]
        },
        "push": {
          "branches": [
            "main"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/ios-build-test.yml",
      "on": {
        "workflow_dispatch": {
          "inputs": {
            "sprint_id": {
              "description": "Sprint identifier for tracking (e.g., sprint-42)",
              "required": "true",
              "type": "string"
            },
            "git_branch": {
              "description": "Branch to build and test",
              "required": "true",
              "default": "main",
              "type": "string"
            }
          }
        },
        "pull_request": {
          "paths": [
            "iOS/**",
            "Cargo.toml",
            "Cargo.lock",
            "core/src/transport/**",
            "core/src/routing/**",
            "core/src/api.udl",
            "core/src/bin/gen_swift.rs",
            "core/src/contacts_bridge.rs",
            "core/src/iron_core.rs",
            "core/src/mobile_bridge.rs",
            "core/src/store/inbox.rs",
            "core/src/store/ledger_entry.rs",
            "core/src/store/storage.rs",
            "mobile/**"
          ]
        },
        "push": {
          "branches": [
            "main"
          ],
          "paths": [
            "iOS/**",
            "Cargo.toml",
            "Cargo.lock",
            "core/src/transport/**",
            "core/src/routing/**",
            "core/src/api.udl",
            "core/src/bin/gen_swift.rs",
            "core/src/contacts_bridge.rs",
            "core/src/iron_core.rs",
            "core/src/mobile_bridge.rs",
            "core/src/store/inbox.rs",
            "core/src/store/ledger_entry.rs",
            "core/src/store/storage.rs",
            "mobile/**"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/lint.yml",
      "on": {
        "pull_request": {
          "branches": [
            "main"
          ]
        },
        "push": {
          "branches": [
            "main"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/mobile.yml",
      "on": {
        "workflow_dispatch": "",
        "push": {
          "branches": [
            "main"
          ],
          "paths": [
            "android/**",
            "iOS/**",
            "core/**",
            "Cargo.toml",
            "Cargo.lock",
            "scripts/build_xcframework.sh",
            "scripts/check_wiring.py",
            "scripts/test_check_wiring.py",
            "scripts/check_unbounded_runblocking.py",
            "scripts/test_check_unbounded_runblocking.py",
            "core/src/api.udl",
            "mobile/**",
            "cli/Cargo.toml",
            "wasm/Cargo.toml",
            "desktop_bridge/Cargo.toml",
            "rust-toolchain.toml",
            ".cargo/**"
          ]
        },
        "release": {
          "types": [
            "published"
          ]
        },
        "pull_request": {
          "paths": [
            "android/**",
            "iOS/**",
            "core/**",
            "Cargo.toml",
            "Cargo.lock",
            "scripts/build_xcframework.sh",
            "scripts/check_wiring.py",
            "scripts/test_check_wiring.py",
            "scripts/check_unbounded_runblocking.py",
            "scripts/test_check_unbounded_runblocking.py",
            "core/src/api.udl",
            "mobile/**",
            "cli/Cargo.toml",
            "wasm/Cargo.toml",
            "desktop_bridge/Cargo.toml",
            "rust-toolchain.toml",
            ".cargo/**",
            "!**.md"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/orchestration-dispatch.yml",
      "on": {
        "workflow_dispatch": {
          "inputs": {
            "git_branch": {
              "description": "Branch that must resolve to candidate_sha",
              "required": "true",
              "type": "string"
            },
            "candidate_sha": {
              "description": "Full lowercase commit SHA to verify and test",
              "required": "true",
              "type": "string"
            }
          }
        }
      }
    },
    {
      "path": ".github/workflows/release.yml",
      "on": {
        "push": {
          "tags": [
            "v*"
          ]
        },
        "workflow_dispatch": {
          "inputs": {
            "artifacts_only": {
              "description": "Build checksummed candidate artifacts without creating a GitHub Release or requiring a tag",
              "required": "false",
              "type": "boolean",
              "default": "false"
            }
          }
        }
      }
    },
    {
      "path": ".github/workflows/scm-test-diagnose-fix.lock.yml",
      "on": {
        "schedule": [
          {
            "cron": "29 2 * * *"
          }
        ],
        "workflow_dispatch": ""
      }
    },
    {
      "path": ".github/workflows/security-regression-tests.yml",
      "on": {
        "push": {
          "branches": [
            "main"
          ]
        },
        "pull_request": {
          "branches": [
            "main"
          ]
        }
      }
    },
    {
      "path": ".github/workflows/security.yml",
      "on": {
        "schedule": [
          {
            "cron": "0 0 * * 0"
          }
        ],
        "workflow_dispatch": ""
      }
    },
    {
      "path": ".github/workflows/stale-branch-sweep.yml",
      "on": {
        "schedule": [
          {
            "cron": "0 9 * * 1"
          }
        ],
        "workflow_dispatch": ""
      }
    },
    {
      "path": ".github/workflows/stale.yml",
      "on": {
        "schedule": [
          {
            "cron": "0 0 * * 0"
          }
        ],
        "workflow_dispatch": ""
      }
    }
  ]
}
```
