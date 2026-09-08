# v0.4.0 architecture boundaries

Status: current structure after the architecture pass.

## Ownership and data flow

- `IronCore` (`core/src/iron_core.rs`) owns subsystem lifetimes and cross-subsystem wiring. It is the only public core entry point; it does not own the internal policy of each subsystem.
- `core/src/store/` owns persistent message, contact, history, and ledger state. Transport and routing consume store APIs rather than opening storage directly.
- `core/src/transport/` owns connectivity state and transport events. `AddressObserver` owns observed-address state and its listener-port admission policy; `TransportManager` owns transport selection, queues, and reconnection state.
- `core/src/routing/` owns route decisions. `LocalCell` owns local peer topology and the shared reliability ordering policy; `RoutingEngine` composes local, neighborhood, and global layers; `OptimizedRoutingEngine` adds caches and scheduling around that decision engine.
- Platform bridges (`cli/`, Android, iOS) translate platform input into core calls and render returned snapshots. They do not become alternate owners of core transport, ledger, or routing state.
- Android UI state belongs in ViewModels/repositories; composables render state and emit user intents. Formatting is pure: `DiagnosticsBundleFormatter` transforms `DiagnosticsBundleInput` into text without owning runtime state.

## Deliberate simplifications

- Address admission is one policy in `AddressObserver`, reused by all promotion paths through its filtered consensus. The empty listener set fails closed.
- Local peer selection uses one deterministic reliability comparator for both hint-specific and all-active queries.
- `OptimizedRoutingEngine` does not duplicate identity or hint state already owned by `RoutingEngine`.

## Flow

```text
platform input -> bridge/repository -> IronCore -> store/transport/routing state
state changes -> subsystem snapshots/events -> IronCore delegate/bridge -> ViewModel -> composable
```

This document describes boundaries, not a new runtime API. Future feature work should put behavior beside the state it governs and add an adapter only when a platform contract requires one.
