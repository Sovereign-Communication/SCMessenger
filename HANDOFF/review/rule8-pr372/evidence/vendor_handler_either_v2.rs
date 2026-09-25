// Copyright 2021 Protocol Labs.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::task::{Context, Poll};

use either::Either;
use futures::future;

use crate::{
    handler::{
        ConnectionEvent, ConnectionHandler, ConnectionHandlerEvent, FullyNegotiatedInbound,
        InboundUpgradeSend, ListenUpgradeError, SubstreamProtocol,
    },
    upgrade::SendWrapper,
};

impl<LIP, RIP, LIOI, RIOI>
    FullyNegotiatedInbound<Either<SendWrapper<LIP>, SendWrapper<RIP>>, Either<LIOI, RIOI>>
where
    RIP: InboundUpgradeSend,
    LIP: InboundUpgradeSend,
{
    // VENDORED PATCH POLICY (SCMessenger D9) -- fatal vs drop, and why.
    //
    // Upstream treats every `Either` side desync as `unreachable!()`, which kills
    // the tokio worker task carrying the connection. Twelve such panics were
    // observed in the field, so the sites are converted rather than left to
    // fire. The classification used throughout this file:
    //
    //   DROP (logged at warn!)  -- a desync that discards ONE event or ONE
    //     upgrade for one connection. The connection itself stays in a defined
    //     state and the peer can re-dial; the cost of a wrong guess is one
    //     lost upgrade, not a dead task or a desynchronised pool.
    //   PROPAGATE AS ERROR       -- a desync that would leave a connection's
    //     protocol set disagreeing with the pool's. No such site exists here;
    //     the one remaining hard failure, `ProtocolsChange::Added` in
    //     handler.rs, is a TEST-ONLY helper behind
    //     `#[cfg(all(test, feature = "upstream-tests"))]` and never compiles in
    //     this workspace, so it is not a production inconsistency.
    //
    // Every drop below is at `warn!`, not `debug!`: a silently discarded
    // fully-negotiated upgrade is a state divergence between two tasks, and it
    // must be visible at default log levels for the field evidence that
    // motivated this patch to be reproducible after the fact.
    //
    // Residual, deliberately NOT solved here: whether a remote peer can induce
    // a desync (rather than local ordering causing it) is an open question in
    // HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md. If it can, the DROP class
    // above must be re-derived.
    // VENDORED PATCH (SCMessenger D9): returns Option so a protocol/info side desync
    // is reported to the caller instead of panicking (`_ => unreachable!()` upstream).
    // Panics were observed live on stock 0.48.0 right after identify.
    pub(crate) fn transpose(
        self,
    ) -> Option<Either<FullyNegotiatedInbound<LIP, LIOI>, FullyNegotiatedInbound<RIP, RIOI>>> {
        match self {
            FullyNegotiatedInbound {
                protocol: future::Either::Left(protocol),
                info: Either::Left(info),
            } => Some(Either::Left(FullyNegotiatedInbound { protocol, info })),
            FullyNegotiatedInbound {
                protocol: future::Either::Right(protocol),
                info: Either::Right(info),
            } => Some(Either::Right(FullyNegotiatedInbound { protocol, info })),
            _ => {
                tracing::warn!(
                    "D9-DEGRADE: Either FullyNegotiatedInbound protocol/info side mismatch; dropping"
                );
                None
            }
        }
    }
}

impl<LIP, RIP, LIOI, RIOI>
    ListenUpgradeError<Either<LIOI, RIOI>, Either<SendWrapper<LIP>, SendWrapper<RIP>>>
where
    RIP: InboundUpgradeSend,
    LIP: InboundUpgradeSend,
{
    // VENDORED PATCH (SCMessenger D9): Option instead of `unreachable!()` on side desync.
    fn transpose(
        self,
    ) -> Option<Either<ListenUpgradeError<LIOI, LIP>, ListenUpgradeError<RIOI, RIP>>> {
        match self {
            ListenUpgradeError {
                error: Either::Left(error),
                info: Either::Left(info),
            } => Some(Either::Left(ListenUpgradeError { error, info })),
            ListenUpgradeError {
                error: Either::Right(error),
                info: Either::Right(info),
            } => Some(Either::Right(ListenUpgradeError { error, info })),
            _ => {
                tracing::warn!(
                    "D9-DEGRADE: Either ListenUpgradeError error/info side mismatch; dropping"
                );
                None
            }
        }
    }
}

/// Implementation of a [`ConnectionHandler`] that represents either of two [`ConnectionHandler`]
/// implementations.
impl<L, R> ConnectionHandler for Either<L, R>
where
    L: ConnectionHandler,
    R: ConnectionHandler,
{
    type FromBehaviour = Either<L::FromBehaviour, R::FromBehaviour>;
    type ToBehaviour = Either<L::ToBehaviour, R::ToBehaviour>;
    type InboundProtocol = Either<SendWrapper<L::InboundProtocol>, SendWrapper<R::InboundProtocol>>;
    type OutboundProtocol =
        Either<SendWrapper<L::OutboundProtocol>, SendWrapper<R::OutboundProtocol>>;
    type InboundOpenInfo = Either<L::InboundOpenInfo, R::InboundOpenInfo>;
    type OutboundOpenInfo = Either<L::OutboundOpenInfo, R::OutboundOpenInfo>;

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        match self {
            Either::Left(a) => a
                .listen_protocol()
                .map_upgrade(|u| Either::Left(SendWrapper(u)))
                .map_info(Either::Left),
            Either::Right(b) => b
                .listen_protocol()
                .map_upgrade(|u| Either::Right(SendWrapper(u)))
                .map_info(Either::Right),
        }
    }

    fn on_behaviour_event(&mut self, event: Self::FromBehaviour) {
        match (self, event) {
            (Either::Left(handler), Either::Left(event)) => handler.on_behaviour_event(event),
            (Either::Right(handler), Either::Right(event)) => handler.on_behaviour_event(event),
            // VENDORED PATCH (SCMessenger D9): was `unreachable!()`; panics observed live
            // on stock 0.48.0 (tokio worker task died on this line after identify). A
            // desynced FromBehaviour event is dropped with a warning instead.
            _ => {
                tracing::warn!(
                    "D9-DEGRADE: Either on_behaviour_event event/handler side mismatch; dropping event"
                );
            }
        }
    }

    fn connection_keep_alive(&self) -> bool {
        match self {
            Either::Left(handler) => handler.connection_keep_alive(),
            Either::Right(handler) => handler.connection_keep_alive(),
        }
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<
        ConnectionHandlerEvent<Self::OutboundProtocol, Self::OutboundOpenInfo, Self::ToBehaviour>,
    > {
        let event = match self {
            Either::Left(handler) => futures::ready!(handler.poll(cx))
                .map_custom(Either::Left)
                .map_protocol(|p| Either::Left(SendWrapper(p)))
                .map_outbound_open_info(Either::Left),
            Either::Right(handler) => futures::ready!(handler.poll(cx))
                .map_custom(Either::Right)
                .map_protocol(|p| Either::Right(SendWrapper(p)))
                .map_outbound_open_info(Either::Right),
        };

        Poll::Ready(event)
    }

    fn poll_close(&mut self, cx: &mut Context<'_>) -> Poll<Option<Self::ToBehaviour>> {
        let event = match self {
            Either::Left(handler) => futures::ready!(handler.poll_close(cx)).map(Either::Left),
            Either::Right(handler) => futures::ready!(handler.poll_close(cx)).map(Either::Right),
        };

        Poll::Ready(event)
    }

    fn on_connection_event(
        &mut self,
        event: ConnectionEvent<
            Self::InboundProtocol,
            Self::OutboundProtocol,
            Self::InboundOpenInfo,
            Self::OutboundOpenInfo,
        >,
    ) {
        match event {
            ConnectionEvent::FullyNegotiatedInbound(fully_negotiated_inbound) => {
                match (fully_negotiated_inbound.transpose(), self) {
                    (Some(Either::Left(fni)), Either::Left(handler)) => {
                        handler.on_connection_event(ConnectionEvent::FullyNegotiatedInbound(fni))
                    }
                    (Some(Either::Right(fni)), Either::Right(handler)) => {
                        handler.on_connection_event(ConnectionEvent::FullyNegotiatedInbound(fni))
                    }
                    // VENDORED PATCH (SCMessenger D9): was `_ => unreachable!()`.
                    (None, _) | (Some(_), _) => tracing::warn!(
                        "D9-DEGRADE: Either FullyNegotiatedInbound handler side mismatch; dropping"
                    ),
                }
            }
            ConnectionEvent::FullyNegotiatedOutbound(fully_negotiated_outbound) => {
                match (fully_negotiated_outbound.transpose(), self) {
                    (Either::Left(fully_negotiated_outbound), Either::Left(handler)) => handler
                        .on_connection_event(ConnectionEvent::FullyNegotiatedOutbound(
                            fully_negotiated_outbound,
                        )),
                    (Either::Right(fully_negotiated_outbound), Either::Right(handler)) => handler
                        .on_connection_event(ConnectionEvent::FullyNegotiatedOutbound(
                            fully_negotiated_outbound,
                        )),
                    // VENDORED PATCH (SCMessenger D9): was `_ => unreachable!()`.
                    _ => tracing::warn!(
                        "D9-DEGRADE: Either FullyNegotiatedOutbound handler side mismatch; dropping"
                    ),
                }
            }
            ConnectionEvent::DialUpgradeError(dial_upgrade_error) => {
                match (dial_upgrade_error.transpose(), self) {
                    (Either::Left(dial_upgrade_error), Either::Left(handler)) => handler
                        .on_connection_event(ConnectionEvent::DialUpgradeError(dial_upgrade_error)),
                    (Either::Right(dial_upgrade_error), Either::Right(handler)) => handler
                        .on_connection_event(ConnectionEvent::DialUpgradeError(dial_upgrade_error)),
                    // VENDORED PATCH (SCMessenger D9): was `_ => unreachable!()`.
                    _ => tracing::warn!(
                        "D9-DEGRADE: Either DialUpgradeError handler side mismatch; dropping"
                    ),
                }
            }
            ConnectionEvent::ListenUpgradeError(listen_upgrade_error) => {
                match (listen_upgrade_error.transpose(), self) {
                    (Some(Either::Left(lue)), Either::Left(handler)) => {
                        handler.on_connection_event(ConnectionEvent::ListenUpgradeError(lue))
                    }
                    (Some(Either::Right(lue)), Either::Right(handler)) => {
                        handler.on_connection_event(ConnectionEvent::ListenUpgradeError(lue))
                    }
                    // VENDORED PATCH (SCMessenger D9): was `_ => unreachable!()`.
                    (None, _) | (Some(_), _) => tracing::warn!(
                        "D9-DEGRADE: Either ListenUpgradeError handler side mismatch; dropping"
                    ),
                }
            }
            ConnectionEvent::AddressChange(address_change) => match self {
                Either::Left(handler) => {
                    handler.on_connection_event(ConnectionEvent::AddressChange(address_change))
                }
                Either::Right(handler) => {
                    handler.on_connection_event(ConnectionEvent::AddressChange(address_change))
                }
            },
            ConnectionEvent::LocalProtocolsChange(supported_protocols) => match self {
                Either::Left(handler) => handler.on_connection_event(
                    ConnectionEvent::LocalProtocolsChange(supported_protocols),
                ),
                Either::Right(handler) => handler.on_connection_event(
                    ConnectionEvent::LocalProtocolsChange(supported_protocols),
                ),
            },
            ConnectionEvent::RemoteProtocolsChange(supported_protocols) => match self {
                Either::Left(handler) => handler.on_connection_event(
                    ConnectionEvent::RemoteProtocolsChange(supported_protocols),
                ),
                Either::Right(handler) => handler.on_connection_event(
                    ConnectionEvent::RemoteProtocolsChange(supported_protocols),
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dummy;
    use libp2p_core::upgrade::{DeniedUpgrade, InboundUpgrade, UpgradeInfo};
    use std::{convert::Infallible, iter};

    /// Minimal inbound upgrade whose `Output` is constructible, so the test can
    /// build `FullyNegotiatedInbound` values directly.
    struct UnitUpgrade;

    // UpgradeInfoSend is blanket-implemented for all `UpgradeInfo + Send + 'static`.
    impl UpgradeInfo for UnitUpgrade {
        type Info = &'static str;
        type InfoIter = iter::Once<&'static str>;

        fn protocol_info(&self) -> Self::InfoIter {
            iter::once("/scm-test/1.0.0")
        }
    }

    impl InboundUpgrade<crate::Stream> for UnitUpgrade {
        type Output = ();
        type Error = Infallible;
        type Future = futures::future::Ready<Result<(), Infallible>>;

        fn upgrade_inbound(self, _socket: crate::Stream, _info: Self::Info) -> Self::Future {
            futures::future::ready(Ok(()))
        }
    }

    /// Handler with a constructible `FromBehaviour` (`()`) — the dummy handler's
    /// `FromBehaviour` is `Infallible`, which cannot force a side desync.
    struct UnitHandler;

    impl ConnectionHandler for UnitHandler {
        type FromBehaviour = ();
        type ToBehaviour = ();
        type InboundProtocol = DeniedUpgrade;
        type OutboundProtocol = DeniedUpgrade;
        type InboundOpenInfo = ();
        type OutboundOpenInfo = ();

        fn listen_protocol(
            &self,
        ) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
            SubstreamProtocol::new(DeniedUpgrade, ())
        }

        fn on_behaviour_event(&mut self, _event: Self::FromBehaviour) {}

        fn poll(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<
            ConnectionHandlerEvent<
                Self::OutboundProtocol,
                Self::OutboundOpenInfo,
                Self::ToBehaviour,
            >,
        > {
            Poll::Pending
        }

        fn poll_close(&mut self, _cx: &mut Context<'_>) -> Poll<Option<Self::ToBehaviour>> {
            Poll::Ready(None)
        }

        fn on_connection_event(
            &mut self,
            _event: ConnectionEvent<Self::InboundProtocol, Self::OutboundProtocol>,
        ) {
        }
    }

    /// REGRESSION (SCMessenger D9): upstream 0.48.0 panics with `unreachable!()`
    /// when a `FullyNegotiatedInbound`'s protocol and info sides disagree. The
    /// patched code must degrade to `None` instead of panicking.
    #[test]
    fn transpose_desync_returns_none_instead_of_panicking() {
        let event: FullyNegotiatedInbound<
            Either<SendWrapper<UnitUpgrade>, SendWrapper<UnitUpgrade>>,
            Either<(), ()>,
        > = FullyNegotiatedInbound {
            // `protocol` is the upgraded Output (`()` for UnitUpgrade), not the upgrade.
            protocol: future::Either::Left(()),
            info: Either::Right(()), // side desync
        };
        assert!(
            event.transpose().is_none(),
            "side-desynced FullyNegotiatedInbound must degrade to None, not panic (SCMessenger D9)"
        );
    }

    /// Matched sides must still route exactly as upstream does.
    #[test]
    fn transpose_matched_sides_still_route() {
        let event: FullyNegotiatedInbound<
            Either<SendWrapper<UnitUpgrade>, SendWrapper<UnitUpgrade>>,
            Either<(), ()>,
        > = FullyNegotiatedInbound {
            protocol: future::Either::Left(()),
            info: Either::Left(()),
        };
        assert!(event.transpose().is_some());
    }

    /// REGRESSION (SCMessenger D9): the live panic fired on this exact arm — a
    /// `FromBehaviour` event routed to the wrong side of an `Either` handler
    /// tree right after identify. Must drop with a warning, not panic.
    #[test]
    fn on_behaviour_event_side_desync_must_not_panic() {
        let mut handler: Either<dummy::ConnectionHandler, UnitHandler> =
            Either::Left(dummy::ConnectionHandler);
        handler.on_behaviour_event(Either::Right(())); // desync: Left handler, Right event
    }
}
