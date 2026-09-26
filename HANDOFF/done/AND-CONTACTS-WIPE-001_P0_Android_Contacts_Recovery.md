# AND-CONTACTS-WIPE-001: Android Contacts Recovery After QUIC/UDP Update

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 (Critical)
**Platform:** Android
**Status:** DONE
**Source:** docs/historical/MASTER_BUG_TRACKER.md
**Completed:** 2026-03-18

## Problem Description
After deploying QUIC/UDP cellular NAT traversal update, Android contacts were wiped while identity and messages remained intact. This is a data loss regression.

## Root Cause Analysis
The QUIC/UDP bootstrap node changes in MeshRepository.kt may have triggered a database migration or contact store corruption.

## Investigation Required
1. Review changes to MeshRepository.kt for contact-related code paths
2. Check if database schema changes occurred  
3. Examine contact persistence logic for migration edge cases
4. Add contact data preservation tests to prevent future regressions

## Key Files to Examine
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- Contact database schema and migration logic
- QUIC/UDP bootstrap implementation changes

## Expected Outcome
1. Identify root cause of contact data loss
2. Implement data recovery mechanism
3. Add preventive measures for future updates
4. Verify contacts are restored and preserved

## Verification
- Fresh install with QUIC/UDP update should preserve existing contacts
- Contact data should survive app updates and restarts
- No data loss during bootstrap configuration changes