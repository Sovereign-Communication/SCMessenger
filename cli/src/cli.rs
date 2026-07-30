use clap::{Parser, Subcommand};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_init() {
        let cli = Cli::parse_from(["scm", "init", "--name", "TestUser"]);
        assert!(matches!(cli.command, Commands::Init { name: Some(n) } if n == "TestUser"));
    }

    #[test]
    fn test_cli_parse_identity_show() {
        let cli = Cli::parse_from(["scm", "identity"]);
        assert!(matches!(cli.command, Commands::Identity { action: None }));
    }

    #[test]
    fn test_cli_parse_contact_add() {
        let cli = Cli::parse_from([
            "scm",
            "contact",
            "add",
            "12D3KooTest12345678901234567890123456789012345678901234",
            "abcd1234efgh5678",
            "--name",
            "Alice",
        ]);
        assert!(
            matches!(cli.command, Commands::Contact { action: ContactAction::Add { peer_id, public_key: _, name: Some(n) } } if peer_id.starts_with("12D3Koo") && n == "Alice")
        );
    }

    #[test]
    fn test_cli_parse_contact_list() {
        let cli = Cli::parse_from(["scm", "contact", "list"]);
        assert!(matches!(
            cli.command,
            Commands::Contact {
                action: ContactAction::List
            }
        ));
    }

    #[test]
    fn test_cli_parse_block_add() {
        let cli = Cli::parse_from(["scm", "block", "add", "test-peer-id"]);
        assert!(
            matches!(cli.command, Commands::Block { action: BlockAction::Add { peer_id, .. } } if peer_id == "test-peer-id")
        );
    }

    #[test]
    fn test_cli_parse_relay() {
        let cli = Cli::parse_from(["scm", "relay", "--listen", "/ip4/127.0.0.1/tcp/9001"]);
        assert!(
            matches!(cli.command, Commands::Relay { listen, .. } if listen == "/ip4/127.0.0.1/tcp/9001")
        );
    }

    #[test]
    fn test_cli_parse_send() {
        let cli = Cli::parse_from(["scm", "send", "recipient-id", "hello world"]);
        assert!(
            matches!(cli.command, Commands::Send { recipient, message } if recipient == "recipient-id" && message == "hello world")
        );
    }

    #[test]
    fn test_cli_parse_status() {
        let cli = Cli::parse_from(["scm", "status"]);
        assert!(matches!(cli.command, Commands::Status));
    }

    #[test]
    fn test_cli_parse_identity_export() {
        let cli = Cli::parse_from([
            "scm",
            "identity",
            "export",
            "--passphrase",
            "secret",
            "--output",
            "backup.json",
        ]);
        assert!(
            matches!(cli.command, Commands::Identity { action: Some(IdentityAction::Export { ref passphrase, output: Some(ref o) }) } if passphrase == "secret" && o == "backup.json")
        );
    }

    #[test]
    fn test_cli_parse_identity_import() {
        let cli = Cli::parse_from([
            "scm",
            "identity",
            "import",
            "--passphrase",
            "secret",
            "--backup",
            "data",
        ]);
        assert!(
            matches!(cli.command, Commands::Identity { action: Some(IdentityAction::Import { ref passphrase, backup: Some(ref b), .. }) } if passphrase == "secret" && b == "data")
        );
    }

    #[test]
    fn test_cli_parse_contact_remove() {
        let cli = Cli::parse_from(["scm", "contact", "remove", "Alice"]);
        assert!(
            matches!(cli.command, Commands::Contact { action: ContactAction::Remove { ref contact } } if contact == "Alice")
        );
    }

    #[test]
    fn test_cli_parse_contact_search() {
        let cli = Cli::parse_from(["scm", "contact", "search", "query"]);
        assert!(
            matches!(cli.command, Commands::Contact { action: ContactAction::Search { ref query } } if query == "query")
        );
    }

    #[test]
    fn test_cli_parse_block_remove() {
        let cli = Cli::parse_from(["scm", "block", "remove", "test-peer-id"]);
        assert!(
            matches!(cli.command, Commands::Block { action: BlockAction::Remove { peer_id, .. } } if peer_id == "test-peer-id")
        );
    }

    #[test]
    fn test_cli_parse_block_delete() {
        let cli = Cli::parse_from(["scm", "block", "delete", "test-peer-id"]);
        assert!(
            matches!(cli.command, Commands::Block { action: BlockAction::Delete { peer_id, .. } } if peer_id == "test-peer-id")
        );
    }
}

const VERSION_INFO: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("SCM_GIT_HASH"),
    " ",
    env!("SCM_BUILD_TIME"),
    ")"
);

#[derive(Parser)]
#[command(name = "scm")]
#[command(about = "SCMessenger — Sovereign Encrypted Messaging", long_about = None)]
#[command(version = VERSION_INFO)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize new identity
    Init {
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Show identity information
    Identity {
        #[command(subcommand)]
        action: Option<IdentityAction>,
    },
    /// Manage contacts
    Contact {
        #[command(subcommand)]
        action: ContactAction,
    },
    /// Configure settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// View message history
    History {
        #[arg(short, long)]
        peer: Option<String>,
        #[arg(short, long)]
        search: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Start P2P messaging node
    Start {
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Run headless relay/bootstrap node (no interactive console)
    Relay {
        /// P2P listen multiaddr (default: /ip4/0.0.0.0/tcp/0)
        #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen: String,
        /// HTTP status/landing page port (default: 9000)
        #[arg(long, default_value = "9000")]
        http_port: u16,
        /// Node name for logging/status (default: auto from peer ID)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Send a message (offline mode)
    Send { recipient: String, message: String },
    /// Show network status
    Status,
    /// Mark an outbox message as delivered/sent
    MarkSent { message_id: String },
    /// Clear all local history records
    HistoryClear {
        /// Required confirmation flag
        #[arg(long)]
        yes: bool,
    },
    /// Keep only the newest N history messages
    HistoryEnforceRetention { max_messages: u32 },
    /// Remove history older than a unix timestamp (seconds)
    HistoryPruneBefore { before_timestamp: u64 },
    /// Stop the running node
    Stop,
    /// Manage peer blocking
    Block {
        #[command(subcommand)]
        action: BlockAction,
    },
    /// Get a single history message by ID
    HistoryGet { id: String },
    /// Show history statistics
    HistoryStats,
    /// Show total history message count
    HistoryCount,
    /// Mark a history message as delivered
    HistoryMarkDelivered { id: String },
    /// Clear conversation history with a specific peer
    HistoryClearConversation { peer: String },
    /// Remove an entire conversation (alias for clear-conversation)
    HistoryRemoveConversation { peer: String },
    /// Delete a single history message by ID
    HistoryDelete { id: String },
    /// Run self-tests
    Test,
    /// Swarm management commands
    Swarm {
        #[command(subcommand)]
        action: SwarmAction,
    },
    /// Manage local network discovery (mDNS, BLE, WiFi-Aware)
    Discovery {
        #[command(subcommand)]
        action: DiscoveryAction,
    },
    /// Host APK for remote QR code scan-to-install on local network
    ShareApk {
        /// Optional path to APK file (default: searches local build outputs)
        #[arg(short, long)]
        apk_path: Option<String>,
        /// Duration in minutes to host server (default: 15)
        #[arg(short, long, default_value = "15")]
        duration_mins: u64,
        /// HTTP port to listen on (default: 8080)
        #[arg(short, long)]
        port: Option<u16>,
    },
}

#[derive(Subcommand)]
pub enum DiscoveryAction {
    /// Show status of all discovery transports
    Status,
    /// Trigger an immediate discovery scan/probe
    Scan {
        /// Optional transport to scan (mdns, ble, wifi-aware)
        #[arg(short, long)]
        transport: Option<String>,
    },
    /// List peers discovered via local transports
    Peers,
}

#[derive(Subcommand)]
pub enum BlockAction {
    /// Block a peer
    Add {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// Unblock a peer
    Remove {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
    },
    /// Block a peer AND delete all their stored messages (cascade purge)
    Delete {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// List all blocked peers
    List,
    /// Check if a peer is blocked
    Check {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
    },
    /// Show total blocked count
    Count,
}

#[derive(Subcommand)]
pub enum IdentityAction {
    Show,
    Export {
        /// Passphrase to encrypt the backup
        #[arg(short, long)]
        passphrase: String,
        /// Optional output file path for backup payload
        #[arg(short, long)]
        output: Option<String>,
    },
    Import {
        /// Passphrase to decrypt the backup
        #[arg(short, long)]
        passphrase: String,
        /// Backup payload string
        #[arg(long, conflicts_with = "input")]
        backup: Option<String>,
        /// Read backup payload from file
        #[arg(short = 'i', long)]
        input: Option<String>,
    },
    SetName {
        name: String,
    },
    /// Show the local device ID (WS13)
    DeviceId,
    /// Show the seniority timestamp (WS13)
    Seniority,
    /// Show registration state for an identity
    RegistrationState {
        identity_id: String,
    },
    /// Sign arbitrary data with the local identity key
    SignData {
        /// Hex-encoded data to sign
        data_hex: String,
    },
    /// Verify a signature against data and a public key
    VerifySignature {
        /// Hex-encoded data
        data_hex: String,
        /// Hex-encoded signature
        signature_hex: String,
        /// Hex-encoded Ed25519 public key
        public_key_hex: String,
    },
}

#[derive(Subcommand)]
pub enum ContactAction {
    Add {
        peer_id: String,
        public_key: String,
        #[arg(short, long)]
        name: Option<String>,
    },
    List,
    Show {
        contact: String,
    },
    Remove {
        contact: String,
    },
    Search {
        query: String,
    },
    SetLocalNickname {
        contact: String,
        nickname: Option<String>,
        #[arg(long)]
        clear: bool,
    },
    /// Set the federated (broadcast) nickname for a contact
    SetNickname {
        contact: String,
        nickname: Option<String>,
        #[arg(long)]
        clear: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    Set { key: String, value: String },
    Get { key: String },
    List,
}

#[derive(Subcommand)]
pub enum SwarmAction {
    /// Show swarm connection statistics
    Stats,
}
