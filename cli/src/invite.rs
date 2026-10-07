// `scm invite create` / `scm invite join` (issue #469 T2).
//
// The only bootstrap source is an invite: a signed `SCI1:` string minted by
// any node (`create`) and redeemed by another (`join`). Nothing here carries or
// embeds an address; the invite is built from the running node's live
// listeners and verified by `IronCore` on redeem.

use crate::api;
use anyhow::{bail, Context, Result};
use clap::{Subcommand, ValueEnum};
use std::path::{Path, PathBuf};

/// Rendering formats for `--qr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum QrFormat {
    /// Terminal text QR (assumes a dark-background terminal).
    Ascii,
}

#[derive(Subcommand, Debug)]
pub enum InviteAction {
    /// Mint a signed SCI1 invite from the running node's live addresses
    Create {
        /// Invite lifetime in seconds (1..=2592000)
        #[arg(long, default_value_t = api::DEFAULT_INVITE_TTL_SECS)]
        ttl: u64,
        /// Also print a QR code to the terminal
        #[arg(long, value_enum)]
        qr: Option<QrFormat>,
        /// Also write the SCI1 string to this file
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Verify an SCI1 invite, import its addresses and dial the inviter
    Join {
        /// The SCI1:... invite string
        invite: Option<String>,
        /// Read the invite from a file instead
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

/// Pick the invite string from the positional argument or `--file`.
///
/// Exactly one source must be given. For a file, the first non-empty line is
/// used. The result is trimmed; validation of the contents is left to
/// `IronCore::redeem_invite_qr`, which fails closed.
pub fn resolve_invite_input(positional: Option<&str>, file: Option<&Path>) -> Result<String> {
    let raw = match (positional, file) {
        (Some(_), Some(_)) => bail!("give either an invite string or --file, not both"),
        (None, None) => bail!("no invite given: pass an SCI1:... string or --file <path>"),
        (Some(s), None) => s.to_string(),
        (None, Some(path)) => {
            let text = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read invite file {}", path.display()))?;
            text.lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or_default()
                .to_string()
        }
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("invite is empty");
    }
    Ok(trimmed.to_string())
}

/// Render `payload` as a text QR code.
pub fn render_qr_ascii(payload: &str) -> Result<String> {
    let code = qrcode::QrCode::new(payload.as_bytes())
        .map_err(|e| anyhow::anyhow!("invite does not fit in a QR code: {}", e))?;
    Ok(code
        .render::<char>()
        .quiet_zone(true)
        .module_dimensions(2, 1)
        .dark_color(' ')
        .light_color('#')
        .build())
}

/// Entry point for the `invite` subcommand.
pub async fn run(action: InviteAction) -> Result<()> {
    match action {
        InviteAction::Create { ttl, qr, out } => create(ttl, qr, out).await,
        InviteAction::Join { invite, file } => {
            let payload = resolve_invite_input(invite.as_deref(), file.as_deref())?;
            join(&payload).await
        }
    }
}

async fn create(ttl: u64, qr: Option<QrFormat>, out: Option<PathBuf>) -> Result<()> {
    if !api::is_api_available().await {
        bail!(
            "no SCMessenger node is running: an invite embeds this node's live \
             addresses, so start one first (`scm start` or `scm relay`)"
        );
    }
    let payload = api::create_invite_via_api(ttl).await?;

    // stdout carries only the invite (plus the optional QR) so scripts can
    // capture the first line.
    println!("{}", payload);

    if let Some(path) = out {
        std::fs::write(&path, format!("{}\n", payload))
            .with_context(|| format!("failed to write {}", path.display()))?;
        eprintln!("[OK] invite written to {}", path.display());
    }
    if let Some(QrFormat::Ascii) = qr {
        println!("{}", render_qr_ascii(&payload)?);
    }
    eprintln!("[INVITE] created ttl_secs={}", ttl);
    Ok(())
}

async fn join(payload: &str) -> Result<()> {
    if api::is_api_available().await {
        let report = api::redeem_invite_via_api(payload).await?;
        eprintln!(
            "[INVITE] redeemed peer={} addrs={}",
            report
                .inviter_peer_id
                .as_deref()
                .unwrap_or(&report.inviter_id),
            report.addresses_offered
        );
        eprintln!(
            "[OK] imported {} new address(es) as unproven; dialed {}/{}",
            report.addresses_imported, report.dial_succeeded, report.dial_attempted
        );
        return Ok(());
    }

    // No running node: import into the persistent ledger so the node dials the
    // seeds on its next start (`seed_dial` reads the same ledger).
    let storage_path = crate::config::Config::data_dir()?.join("storage");
    let storage = storage_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("storage path is not valid UTF-8"))?
        .to_string();
    let core = scmessenger_core::IronCore::with_storage(storage);
    let report = core
        .redeem_invite_qr(payload.to_string())
        .map_err(|e| anyhow::anyhow!("invite rejected: {}", e))?;
    eprintln!(
        "[OK] no node running: imported {} new address(es) into the ledger; \
         they are dialed when the node starts",
        report.addresses_imported
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmessenger_core::relay::invite::InviteToken;
    use scmessenger_core::IronCore;

    fn core() -> IronCore {
        let core = IronCore::new();
        core.grant_consent();
        core.initialize_identity().expect("identity");
        core
    }

    fn addrs() -> Vec<String> {
        vec!["/ip4/203.0.113.9/tcp/9001".to_string()]
    }

    #[test]
    fn input_requires_exactly_one_source() {
        assert!(resolve_invite_input(None, None).is_err());
        assert!(resolve_invite_input(Some("SCI1:x"), Some(Path::new("f"))).is_err());
        assert!(resolve_invite_input(Some("   "), None).is_err());
        assert_eq!(
            resolve_invite_input(Some("  SCI1:abc \n"), None).expect("ok"),
            "SCI1:abc"
        );
    }

    #[test]
    fn input_from_file_takes_first_nonempty_line() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("invite.txt");
        std::fs::write(&path, "\n  SCI1:fromfile  \nignored\n").expect("write");
        assert_eq!(
            resolve_invite_input(None, Some(&path)).expect("ok"),
            "SCI1:fromfile"
        );
        let missing = dir.path().join("nope.txt");
        assert!(resolve_invite_input(None, Some(&missing)).is_err());
    }

    #[test]
    fn round_trip_via_file_and_qr() {
        let inviter = core();
        let invitee = core();
        let payload = inviter.create_invite_qr(addrs(), 3600).expect("create");

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("invite.txt");
        std::fs::write(&path, format!("{}\n", payload)).expect("write");
        let read_back = resolve_invite_input(None, Some(&path)).expect("read");
        assert_eq!(read_back, payload);

        let report = invitee.redeem_invite_qr(read_back).expect("redeem");
        assert_eq!(report.addresses_imported, 1);

        let qr = render_qr_ascii(&payload).expect("qr");
        assert!(qr.contains('#') && qr.lines().count() > 10);
    }

    #[test]
    fn tampered_malformed_and_expired_are_rejected() {
        let inviter = core();
        let invitee = core();
        let payload = inviter.create_invite_qr(addrs(), 3600).expect("create");

        let mut token = InviteToken::from_qr_payload(&payload).expect("decode");
        token.seed_ledger[0].multiaddr = "/ip4/198.51.100.7/tcp/1".to_string();
        let forged = token.to_qr_payload().expect("encode");
        assert!(invitee.redeem_invite_qr(forged).is_err());

        assert!(invitee.redeem_invite_qr("garbage".to_string()).is_err());
        assert!(invitee.redeem_invite_qr("SCI1:!!".to_string()).is_err());

        let mut expired = InviteToken::from_qr_payload(&payload).expect("decode");
        expired.expires_at = expired.created_at;
        let sig = inviter
            .sign_data(expired.get_signable_data().expect("signable"))
            .expect("sign")
            .signature;
        expired.signature = sig;
        let expired_payload = expired.to_qr_payload().expect("encode");
        assert!(invitee.redeem_invite_qr(expired_payload).is_err());
    }
}
