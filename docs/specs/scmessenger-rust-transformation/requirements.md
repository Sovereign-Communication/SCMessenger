# Requirements Document: SCMessenger Rust Transformation

## Introduction

This document specifies the requirements for a comprehensive transformation of the SCMessenger Rust codebase to address critical safety, architectural, performance, and observability issues identified in a detailed audit. The transformation follows a phased approach across 6 major phases (Phase 0-5), with each phase building upon the previous one. The project maintains a zero-regression requirement: all existing functionality must be preserved throughout the transformation.

SCMessenger is a decentralized peer-to-peer messaging system built with Rust, libp2p, and WASM, supporting multiple platforms (desktop CLI, mobile, web). The transformation addresses 15 critical issues ranging from panic vectors in production code to missing observability infrastructure, while establishing SCMessenger as a reference implementation for Rust P2P systems.

## Glossary

- **Core_Crate**: The scmessenger_core library crate containing mesh networking, sync protocol, and storage logic
- **CLI_Crate**: The scmessenger_cli binary crate providing desktop daemon and HTTP API
- **WASM_Crate**: The scmessenger_wasm crate compiled to WebAssembly for browser integration
- **Mesh_State**: The distributed state of messages and peers across the P2P network
- **IBLT_Sync**: Invertible Bloom Lookup Table synchronization protocol for efficient set reconciliation
- **Drift_Envelope**: Message container used in the mesh sync protocol
- **Panic_Vector**: Code path that can cause runtime panic (unwrap, expect, panic! macros)
- **Async_Context**: Code executing within a tokio async runtime
- **Eviction_Policy**: Strategy for removing items when storage capacity is exceeded
- **Schema_Version**: Protocol version number for forward-compatible serialization
- **Peer_Proof**: Cryptographic hash proving knowledge of current mesh state
- **Rate_Limiter**: Component enforcing maximum operation frequency per peer
- **Property_Based_Test**: Test using generated inputs to verify invariants hold for all cases
- **Tracing_Framework**: Structured logging system with span-based context propagation

## Requirements

### Requirement 1: Eliminate Panic Vectors in Production Code

**User Story:** As a system operator, I want the application to handle all errors gracefully without panicking, so that the service remains available even when encountering unexpected conditions.

#### Acceptance Criteria

1. THE Core_Crate SHALL NOT contain .unwrap() or .expect() calls in non-test code paths
2. THE CLI_Crate SHALL NOT contain .unwrap() or .expect() calls in non-test code paths
3. THE WASM_Crate SHALL NOT contain .unwrap() or .expect() calls in non-test code paths
4. WHEN UniFFI scaffolding generation fails, THE build process SHALL exit with a descriptive error message
5. WHEN a lock acquisition fails, THE system SHALL propagate the error using Result types
6. WHEN parsing or deserialization fails, THE system SHALL return a typed error variant
7. FOR ALL production code paths, panic! and unreachable! macros SHALL be justified with SAFETY comments or removed

### Requirement 2: Establish Structured Error Hierarchy

**User Story:** As a library consumer, I want typed error variants with context, so that I can handle different error conditions appropriately.

#### Acceptance Criteria

1. THE Core_Crate SHALL define error types using thiserror derive macros
2. THE Core_Crate SHALL provide a MeshError enum covering all mesh operation failures
3. THE MeshError enum SHALL include variants for: version mismatch, transport failure, relay denial, storage quota, serialization failure, authentication failure, and IBLT decode failure
4. THE Core_Crate SHALL provide a TransportError enum for network-layer failures
5. THE Core_Crate SHALL provide a SerializationError enum for encoding/decoding failures
6. WHEN an error occurs in Core_Crate, THE system SHALL return a typed error variant with relevant context fields
7. THE Core_Crate SHALL NOT use anyhow for error types (library crates must expose structured errors)
8. FOR ALL error variants, the Display implementation SHALL provide human-readable messages with context

### Requirement 3: Standardize Lock Usage for Async Contexts

**User Story:** As a performance engineer, I want async-aware locks in async contexts, so that blocking operations don't stall the executor thread.

#### Acceptance Criteria

1. WHEN code executes in Async_Context, THE system SHALL use tokio::sync::Mutex or tokio::sync::RwLock
2. WHEN code executes in synchronous contexts, THE system SHALL use parking_lot or std::sync locks
3. WHEN code executes in WASM (single-threaded), THE system SHALL use RefCell or Cell instead of locks
4. THE CLI_Crate SHALL NOT use std::sync::Mutex or parking_lot::Mutex in async function bodies
5. THE WASM_Crate SHALL NOT use parking_lot::RwLock (WASM is single-threaded)
6. WHEN a lock is held across an await point, THE system SHALL use tokio::sync lock types
7. FOR ALL async functions, clippy::await_holding_lock warnings SHALL be resolved

### Requirement 4: Migrate HTTP API from Hyper 0.14 to Axum 0.7

**User Story:** As a maintainer, I want to use a supported HTTP framework with type-safe routing, so that the API is maintainable and secure.

#### Acceptance Criteria

1. THE CLI_Crate SHALL use Axum 0.7 for HTTP routing instead of Hyper 0.14
2. THE CLI_Crate SHALL define routes using Axum's type-safe extractors (State, Path, Json)
3. THE CLI_Crate SHALL implement CORS middleware using tower-http
4. THE CLI_Crate SHALL NOT have Hyper 0.14 in its dependency tree
5. WHEN the API server starts, THE system SHALL bind to the configured address using tokio::net::TcpListener
6. WHEN a request is received, THE system SHALL route it using Axum's Router
7. FOR ALL API endpoints (/send, /identity, /peers, /messages/:peer_id), functionality SHALL be preserved after migration

### Requirement 5: Add Schema Versioning to Network Messages

**User Story:** As a network operator, I want protocol version negotiation, so that nodes can upgrade asynchronously without breaking the mesh.

#### Acceptance Criteria

1. THE Core_Crate SHALL define a SYNC_SCHEMA_VERSION constant
2. THE Core_Crate SHALL wrap all SyncMessage variants in a VersionedSyncMessage container
3. THE VersionedSyncMessage SHALL include a schema_version field
4. WHEN deserializing a VersionedSyncMessage, THE system SHALL validate the schema_version matches SYNC_SCHEMA_VERSION
5. IF the schema_version does not match, THEN THE system SHALL return MeshError::VersionMismatch with got and expected versions
6. THE SyncMessage enum SHALL use serde tag attribute for forward-compatible variant encoding
7. FOR ALL network message types, serialization SHALL round-trip correctly with both old and new schema versions during migration

### Requirement 6: Add Cryptographic Peer Proofs to Sync Protocol

**User Story:** As a security engineer, I want sync offers to prove knowledge of mesh state, so that attackers cannot trigger expensive diff computations blindly.

#### Acceptance Criteria

1. THE Mesh_State SHALL provide a generate_proof method returning a blake3 hash
2. THE generate_proof method SHALL hash all message IDs and timestamps in the current state
3. THE SyncOffer message SHALL include a peer_proof field containing the blake3 hash
4. THE SyncOffer message SHALL include a timestamp field to prevent replay attacks
5. WHEN receiving a SyncOffer, THE system SHALL validate the peer_proof before computing IBLT diff
6. IF the peer_proof is invalid, THEN THE system SHALL reject the sync request with MeshError::Auth
7. FOR ALL identical Mesh_State instances, generate_proof SHALL produce identical hashes (deterministic)

### Requirement 7: Implement Rate Limiting for Sync Initiations

**User Story:** As a node operator, I want to limit sync request frequency per peer, so that malicious peers cannot DoS my node with sync floods.

#### Acceptance Criteria

1. THE Core_Crate SHALL provide a SyncRateLimiter component
2. THE SyncRateLimiter SHALL track sync request timestamps per peer within a sliding time window
3. THE SyncRateLimiter SHALL enforce a maximum number of sync requests per peer per time window
4. WHEN a peer exceeds the rate limit, THE system SHALL deny the sync request with MeshError::RateLimited
5. THE SyncRateLimiter SHALL automatically purge expired timestamp entries to prevent unbounded growth
6. THE SyncRateLimiter SHALL use tokio::sync::Mutex for async-safe state access
7. FOR ALL peers, rate limit state SHALL be isolated (one peer's limit does not affect others)

### Requirement 8: Implement Bounded Storage with Eviction Policies

**User Story:** As a WASM application developer, I want automatic message eviction when storage is full, so that the browser doesn't run out of memory.

#### Acceptance Criteria

1. THE WASM_Crate SHALL provide a BoundedStorage component with configurable capacity
2. THE BoundedStorage SHALL support multiple Eviction_Policy variants: OldestFirst, UnknownSendersFirst, LargestFirst
3. WHEN pushing a message to full storage, THE BoundedStorage SHALL evict one message according to the configured policy before inserting
4. THE BoundedStorage SHALL track current_size in bytes for byte-weighted capacity limits
5. THE BoundedStorage SHALL NOT exceed the configured capacity at any time
6. WHEN evicting a message, THE BoundedStorage SHALL update current_size to reflect the freed space
7. FOR ALL eviction policies, the storage SHALL maintain correct size accounting (no leaks or underflows)

### Requirement 9: Optimize Message Deduplication with Fixed-Size Hashes

**User Story:** As a performance engineer, I want efficient deduplication with minimal memory overhead, so that the system scales to tens of thousands of messages.

#### Acceptance Criteria

1. THE Core_Crate inbox SHALL use blake3 hashes of message IDs instead of storing full String keys
2. THE inbox deduplication set SHALL use FxHashSet<[u8; 32]> for fixed-size hash storage
3. THE inbox SHALL provide is_duplicate method accepting a message ID string
4. THE inbox SHALL provide mark_seen method accepting a message ID string
5. WHEN the deduplication set reaches max_size, THE system SHALL evict entries (FIFO or periodic clear)
6. THE deduplication logic SHALL NOT produce false negatives (must catch all actual duplicates)
7. FOR ALL message IDs, the blake3 hash SHALL be computed consistently

### Requirement 10: Optimize WASM Bundle Size

**User Story:** As a web developer, I want a small WASM bundle, so that the application loads quickly in browsers.

#### Acceptance Criteria

1. THE WASM_Crate SHALL use Cargo profile with opt-level="z", lto=true, codegen-units=1, panic="abort"
2. THE WASM build process SHALL run wasm-opt with -Oz flag after wasm-pack build
3. THE WASM_Crate SHALL measure and record bundle size before and after optimization
4. THE WASM_Crate build script SHALL verify wasm-opt is available or provide installation instructions
5. WHEN building for release, THE system SHALL apply all size optimizations automatically
6. THE optimized WASM bundle SHALL be functionally equivalent to the unoptimized version
7. FOR ALL WASM builds, the optimization process SHALL complete without errors

### Requirement 11: Integrate Tracing Framework Across All Crates

**User Story:** As a system operator, I want structured logging with context propagation, so that I can debug distributed mesh issues effectively.

#### Acceptance Criteria

1. THE Core_Crate SHALL use the tracing crate for all logging
2. THE CLI_Crate SHALL initialize tracing_subscriber with environment-based filtering
3. THE CLI_Crate SHALL NOT use println! or eprintln! in production code (use tracing macros instead)
4. WHEN the CLI starts, THE system SHALL configure tracing output format and filter level from RUST_LOG environment variable
5. THE relay_envelope function SHALL use #[tracing::instrument] with message ID in span fields
6. THE sync protocol functions SHALL use tracing::info, tracing::warn, and tracing::error for events
7. FOR ALL instrumented functions, span context SHALL propagate through async await points

### Requirement 12: Add Property-Based Tests for IBLT Sync

**User Story:** As a protocol developer, I want automated testing of sync correctness across arbitrary message sets, so that edge cases are caught before production.

#### Acceptance Criteria

1. THE Core_Crate SHALL include proptest-based tests for IBLT_Sync protocol
2. THE property test SHALL generate arbitrary local and remote message sets (0-500 messages each)
3. THE property test SHALL execute the full sync protocol: create offer, respond to offer, apply response
4. WHEN sync completes, THE local and remote stores SHALL contain the union of both original sets
5. THE property test SHALL verify no messages are lost during sync
6. THE property test SHALL verify no duplicate messages are introduced during sync
7. FOR ALL generated message set pairs, the sync protocol SHALL converge to identical state (round-trip property)

### Requirement 13: Parse and Pretty-Print Configuration Files

**User Story:** As a developer, I want to parse configuration files into structured objects and format them back, so that configuration can be validated and normalized.

#### Acceptance Criteria

1. WHEN a valid configuration file is provided, THE Parser SHALL parse it into a Configuration object
2. WHEN an invalid configuration file is provided, THE Parser SHALL return a descriptive error with line/column information
3. THE Core_Crate SHALL provide a Pretty_Printer component for Configuration objects
4. THE Pretty_Printer SHALL format Configuration objects back into valid configuration file syntax
5. FOR ALL valid Configuration objects, parsing then printing then parsing SHALL produce an equivalent object (round-trip property)
6. THE Parser SHALL validate all required fields are present
7. THE Parser SHALL reject configuration files with unknown fields or invalid values

### Requirement 14: Document All Public APIs

**User Story:** As a library consumer, I want comprehensive documentation for all public APIs, so that I can integrate SCMessenger without reading implementation code.

#### Acceptance Criteria

1. THE Core_Crate SHALL have doc comments (///) on all pub fn, pub struct, pub enum, and pub trait items
2. THE CLI_Crate SHALL have doc comments on all public API types and functions
3. THE WASM_Crate SHALL have doc comments on all exported functions and types
4. WHEN running cargo doc, THE system SHALL produce zero documentation warnings
5. THE documentation SHALL include usage examples for primary APIs (MeshNode, SyncProtocol, BoundedStorage)
6. THE documentation SHALL describe error conditions and return types
7. FOR ALL public APIs, the documentation SHALL be accurate and up-to-date with implementation

### Requirement 15: Create Architecture Documentation

**User Story:** As a new contributor, I want high-level architecture documentation, so that I can understand system design before diving into code.

#### Acceptance Criteria

1. THE project SHALL include a docs/architecture.md file
2. THE architecture document SHALL describe the mesh topology and peer discovery mechanisms
3. THE architecture document SHALL explain the IBLT sync protocol and set reconciliation algorithm
4. THE architecture document SHALL document the security model including encryption and authentication
5. THE architecture document SHALL describe the WASM bridge and FFI boundaries
6. THE architecture document SHALL include diagrams for message flow and state transitions
7. WHEN running mdbook build, THE documentation SHALL compile without errors

### Requirement 16: Maintain Zero-Regression Throughout Transformation

**User Story:** As a project stakeholder, I want all existing functionality preserved during refactoring, so that users experience no service disruption.

#### Acceptance Criteria

1. WHEN any phase completes, THE system SHALL pass all existing unit tests
2. WHEN any phase completes, THE system SHALL pass all existing integration tests
3. WHEN any phase completes, THE system SHALL compile without errors in all crates (core, cli, wasm)
4. WHEN any phase completes, THE system SHALL pass cargo clippy with -D warnings flag
5. WHEN any phase completes, THE system SHALL pass cargo fmt --check
6. THE transformation SHALL NOT delete public APIs without a deprecation cycle
7. FOR ALL phases, a smoke test (send and receive message) SHALL pass before proceeding to next phase

### Requirement 17: Implement Atomic Multi-Key Storage Updates

**User Story:** As a data integrity engineer, I want atomic updates across multiple storage keys, so that crashes don't leave the database in inconsistent state.

#### Acceptance Criteria

1. THE CLI_Crate contact storage SHALL use sled::transaction for multi-key updates
2. WHEN updating a contact's nickname and public key, THE system SHALL perform both writes in a single transaction
3. IF a transaction fails, THEN THE system SHALL rollback all changes and return an error
4. THE history storage SHALL use transactions for related message and metadata updates
5. THE transaction logic SHALL handle retry on conflict (optimistic concurrency)
6. THE storage layer SHALL NOT have partial updates visible to readers during transaction execution
7. FOR ALL multi-key operations, atomicity SHALL be guaranteed (all-or-nothing)

### Requirement 18: Implement CORS and Authentication for HTTP API

**User Story:** As a security engineer, I want origin validation and request authentication on the HTTP API, so that malicious webpages cannot abuse the local daemon.

#### Acceptance Criteria

1. THE CLI_Crate HTTP API SHALL implement CORS middleware using tower-http
2. THE CORS configuration SHALL allow only configured origins (e.g., http://localhost:9000)
3. THE CORS configuration SHALL allow only GET and POST methods
4. THE HTTP API SHALL implement an API key authentication middleware
5. WHEN a request lacks a valid API key, THE system SHALL return 401 Unauthorized
6. THE API key SHALL be configurable via environment variable or config file
7. FOR ALL API endpoints, CORS headers SHALL be present in responses

### Requirement 19: Remove or Feature-Gate Stub DSPy Module

**User Story:** As a build engineer, I want to exclude unimplemented code from default builds, so that compile times and binary size are minimized.

#### Acceptance Criteria

1. THE Core_Crate SHALL feature-gate the dspy module behind a "dspy" feature flag
2. THE "dspy" feature SHALL be disabled by default
3. WHEN the "dspy" feature is disabled, THE dspy module SHALL NOT be compiled
4. WHEN the "dspy" feature is disabled, THE UniFFI scaffolding for dspy SHALL NOT be generated
5. THE Core_Crate Cargo.toml SHALL document the "dspy" feature and its purpose
6. THE default build SHALL NOT include dspy code or dependencies
7. FOR ALL builds without "dspy" feature, compile time and binary size SHALL be reduced

### Requirement 20: Compress Log Deltas with Variable-Length Encoding

**User Story:** As a storage engineer, I want efficient encoding of timestamp deltas, so that log storage overhead is minimized.

#### Acceptance Criteria

1. THE Core_Crate log storage SHALL use unsigned-varint encoding for timestamp deltas
2. THE serialize_deltas method SHALL encode each u32 delta using variable-length encoding
3. THE deserialize_deltas method SHALL decode variable-length encoded deltas back to u32 values
4. WHEN deltas are small (<128), THE encoding SHALL use 1 byte per delta
5. WHEN deltas are large, THE encoding SHALL use up to 5 bytes per delta
6. THE serialization SHALL round-trip correctly (serialize then deserialize produces original values)
7. FOR ALL delta sequences, the variable-length encoding SHALL reduce storage size compared to fixed 4-byte encoding

