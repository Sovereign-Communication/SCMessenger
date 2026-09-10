#!/usr/bin/env bash
# Verify a release keystore locally BEFORE setting any GitHub secret.
#
# Usage: scripts/verify_release_keystore.sh <keystore-file> <alias>
#
# Prompts once for the store password (never echoed, never stored, never passed
# on the command line where it would land in shell history). Prints the
# certificate fingerprints and whether the alias resolves -- it does NOT print
# the password, and it does NOT print the alias back.
#
# Why this exists: the alias is checked case-sensitively by PKCS12 keystores but
# was case-INSENSITIVE under the legacy JKS format. `docs/ANDROID_RELEASE_SIGNING.md`
# documented `-storetype JKS` while modern keytool writes PKCS12, so an alias that
# "looks right" can still fail. That mismatch cost this project a 24-minute CI
# round trip per attempt until the release.yml preflight was added, and it is the
# reason the v0.4.0-rc.1 release build failed at :app:packageRelease.
#
# Run this, get [OK], and only then set SCMESSENGER_KEY_ALIAS.
set -euo pipefail

KEYSTORE="${1:-}"
ALIAS="${2:-}"

if [ -z "$KEYSTORE" ] || [ -z "$ALIAS" ]; then
  echo "Usage: scripts/verify_release_keystore.sh <keystore-file> <alias>"
  exit 2
fi

if [ ! -f "$KEYSTORE" ]; then
  echo "[FAIL] keystore not found: $KEYSTORE"
  exit 1
fi

command -v keytool >/dev/null 2>&1 || { echo "[FAIL] keytool not on PATH (install a JDK)"; exit 1; }

# -s keeps the password off the screen; it is used only in this process.
read -r -s -p "Keystore password (not echoed): " STOREPASS
echo

echo "[INFO] keystore: $KEYSTORE"
echo "[INFO] store type as written on disk:"
keytool -list -keystore "$KEYSTORE" -storepass "$STOREPASS" > /tmp/.ks_list.$$ 2>&1 || {
  rc=$?
  echo "[FAIL] could not open the keystore -- wrong store password, or not a keystore"
  rm -f /tmp/.ks_list.$$
  exit $rc
}
grep -iE "keystore type|Your keystore contains" /tmp/.ks_list.$$ || true

# The exact-alias check. This is precisely what release.yml's preflight runs.
if keytool -list -keystore "$KEYSTORE" -storepass "$STOREPASS" -alias "$ALIAS" >/dev/null 2>&1; then
  echo "[OK] the alias resolves in this keystore"
else
  echo "[FAIL] that alias is NOT present in this keystore"
  echo "[INFO] aliases actually present (compare CASE carefully):"
  grep -iE "^[^,]+, .*, (PrivateKeyEntry|trustedCertEntry)" /tmp/.ks_list.$$ | sed 's/^/    /' || \
    sed -n '/Your keystore contains/,$p' /tmp/.ks_list.$$ | sed 's/^/    /'
  rm -f /tmp/.ks_list.$$
  exit 1
fi
rm -f /tmp/.ks_list.$$

echo "[INFO] certificate fingerprints for this alias -- record these:"
keytool -list -v -keystore "$KEYSTORE" -storepass "$STOREPASS" -alias "$ALIAS" 2>/dev/null \
  | grep -iE "SHA1:|SHA256:|Valid from|Signature algorithm|Subject:" | sed 's/^/    /'

echo
echo "[OK] verification passed. Safe to set the secrets:"
echo "     gh secret set SCMESSENGER_KEY_ALIAS   # then type the alias exactly as verified above"
echo
echo "[WARNING] Record the SHA-256 fingerprint above in your password manager alongside"
echo "          the keystore. It is how you prove later that a given APK came from this key,"
echo "          and how you detect a silent key swap."
