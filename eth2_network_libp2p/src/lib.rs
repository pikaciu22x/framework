use core::{iter, ops::Deref as _};

use anyhow::{bail, ensure, Error, Result};
use error_utils::{DebugAsError, SyncError};
use eth2_libp2p::{
    rpc::{
        methods::{BeaconBlocksRequest, GoodbyeReason, HelloMessage, RecentBeaconBlocksRequest},
        ErrorMessage, RPCError, RPCErrorResponse, RPCRequest, RPCResponse, RequestId,
    },
    Libp2pEvent, PeerId, PubsubMessage, RPCEvent, Service, Topic, TopicHash,
};
use eth2_network::{Network, Networked, Status};
use ethereum_types::H32;
use fmt_extra::{AsciiStr, Hs};
use futures::{
    future, try_ready,
    unsync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    Async, Future, Poll, Stream as _,
};
use helper_functions::misc;
use log::info;
use slog::{o, Drain as _, Logger};
use slog_stdlog::StdLog;
use ssz::{Decode as _, Encode as _};
use thiserror::Error;
use types::{
    config::Config,
    primitives::Version,
    types::{Attestation, BeaconBlock},
};

pub use eth2_libp2p::NetworkConfig;
pub use qutex::{Guard, Qutex};

#[derive(Debug, Error)]
enum EventHandlerError {
    #[error("error while sending message to peer {peer_id}: {rpc_error:?}")]
    RpcError {
        peer_id: PeerId,
        rpc_error: RPCError,
    },
    #[error(
        "peer {} sent a response to RecentBeaconBlocks without request: {}",
        peer_id,
        Hs(response_bytes)
    )]
    UnexpectedResponse {
        peer_id: PeerId,
        response_bytes: Vec<u8>,
    },
    #[error(
        "peer {} rejected the request: {}",
        peer_id,
        AsciiStr(&error_message.error_message)
    )]
    InvalidRequest {
        peer_id: PeerId,
        error_message: ErrorMessage,
    },
    #[error(
        "peer {} encountered an error: {}",
        peer_id,
        AsciiStr(&error_message.error_message)
    )]
    ServerError {
        peer_id: PeerId,
        error_message: ErrorMessage,
    },
    #[error(
        "peer {} responded with unknown response code: {}",
        peer_id,
        AsciiStr(&error_message.error_message)
    )]
    UnknownResponse {
        peer_id: PeerId,
        error_message: ErrorMessage,
    },
    #[error("unsupported gossiped object type (id: {id:?}, peer_id: {peer_id}, topics: {topics:?}, message: {message:?})")]
    UnsupportedGossipedObjectType {
        id: String,
        // `eth2-libp2p` calls this `source` rather than `peer_id`, but we can't use that name
        // because `thiserror` treats `source` fields specially and provides no way to opt out.
        peer_id: PeerId,
        topics: Vec<TopicHash>,
        message: PubsubMessage,
    },
    #[error("slot step is zero")]
    SlotStepIsZero,
    #[error("slot difference overflowed ({count} * {step})")]
    SlotDifferenceOverflow { count: u64, step: u64 },
    #[error("end slot overflowed ({start_slot} + {difference})")]
    EndSlotOverflow { start_slot: u64, difference: u64 },
    #[error(
        "local fork version ({}) is different from remote fork version ({})",
        H32(*local),
        H32(*remote)
    )]
    ForkVersionMismatch { local: Version, remote: Version },
    #[error("ran out of request IDs")]
    RequestIdsExhausted,
}

#[allow(clippy::large_enum_variant)]
enum Gossip<C: Config> {
    BeaconBlock(BeaconBlock<C>),
    BeaconAttestation(Attestation<C>),
}

pub struct Sender<C: Config>(UnboundedSender<Gossip<C>>);

pub struct Receiver<C: Config>(UnboundedReceiver<Gossip<C>>);

impl<C: Config> Network<C> for Sender<C> {
    fn publish_beacon_block(&self, beacon_block: BeaconBlock<C>) -> Result<()> {
        self.0
            .unbounded_send(Gossip::BeaconBlock(beacon_block))
            .map_err(Into::into)
    }

    fn publish_beacon_attestation(&self, attestation: Attestation<C>) -> Result<()> {
        self.0
            .unbounded_send(Gossip::BeaconAttestation(attestation))
            .map_err(Into::into)
    }
}

type EventFuture = Box<dyn Future<Item = (), Error = Error>>;

struct EventHandler<C: Config, N> {
    networked: Qutex<N>,
    networked_receiver: Receiver<C>,
    // Wrapping `Service` in a `Qutex` isn't strictly necessary but simplifies the types of
    // `in_progress` and `handle_libp2p_event`.
    service: Qutex<Service>,
    next_request_id: usize,
    in_progress: Option<EventFuture>,
}

impl<C: Config, N: Networked<C>> EventHandler<C, N> {
    fn handle_libp2p_event(&mut self, libp2p_event: Libp2pEvent) -> Result<EventFuture> {
        match libp2p_event {
            Libp2pEvent::RPC(peer_id, RPCEvent::Request(request_id, RPCRequest::Hello(hello))) => {
                self.handle_hello_request(peer_id, request_id, hello)
            }
            Libp2pEvent::RPC(peer_id, RPCEvent::Request(_, RPCRequest::Goodbye(reason))) => {
                self.handle_goodbye_request(&peer_id, &reason)
            }
            Libp2pEvent::RPC(
                peer_id,
                RPCEvent::Request(request_id, RPCRequest::BeaconBlocks(request)),
            ) => self.handle_beacon_blocks_request(peer_id, request_id, &request),
            Libp2pEvent::RPC(
                peer_id,
                RPCEvent::Request(request_id, RPCRequest::RecentBeaconBlocks(request)),
            ) => self.handle_recent_beacon_blocks_request(peer_id, request_id, request),
            Libp2pEvent::RPC(peer_id, RPCEvent::Response(_, response)) => {
                self.handle_rpc_response(peer_id, response)
            }
            Libp2pEvent::RPC(peer_id, RPCEvent::Error(_, rpc_error)) => {
                bail!(EventHandlerError::RpcError { peer_id, rpc_error });
            }
            Libp2pEvent::PeerDialed(peer_id) => self.handle_peer_dialed(peer_id),
            Libp2pEvent::PeerDisconnected(peer_id) => {
                info!("peer {} disconnected", peer_id);
                Ok(Box::new(future::ok(())))
            }
            Libp2pEvent::PubsubMessage {
                id,
                source,
                topics,
                message,
            } => self.handle_pubsub_message(id, source, topics, message),
        }
    }

    fn handle_hello_request(
        &mut self,
        peer_id: PeerId,
        hello_request_id: RequestId,
        hello: HelloMessage,
    ) -> Result<EventFuture> {
        let remote = hello_message_into_status(hello);

        info!(
            "received Hello request (peer_id: {}, remote: {:?})",
            peer_id, remote,
        );

        let beacon_blocks_request_id = self.request_id()?;

        Ok(Box::new(
            self.lock_networked().join(self.lock_service()).and_then(
                move |(networked, mut service)| {
                    let local = get_and_check_status(networked.deref(), remote)?;

                    info!(
                        "sending Hello response (peer_id: {}, local: {:?})",
                        peer_id, local,
                    );

                    service.swarm.send_rpc(
                        peer_id.clone(),
                        RPCEvent::Response(
                            hello_request_id,
                            RPCErrorResponse::Success(RPCResponse::Hello(
                                status_into_hello_message(local),
                            )),
                        ),
                    );

                    compare_status_and_request_blocks::<C>(
                        local,
                        remote,
                        service,
                        peer_id,
                        beacon_blocks_request_id,
                    );

                    Ok(())
                },
            ),
        ))
    }

    fn handle_goodbye_request(
        &self,
        peer_id: &PeerId,
        reason: &GoodbyeReason,
    ) -> Result<EventFuture> {
        info!(
            "received Goodbye (peer_id: {}, reason: {})",
            peer_id, reason,
        );
        Ok(Box::new(future::ok(())))
    }

    fn handle_beacon_blocks_request(
        &self,
        peer_id: PeerId,
        request_id: RequestId,
        request: &BeaconBlocksRequest,
    ) -> Result<EventFuture> {
        info!(
            "received BeaconBlocks request (peer_id: {}, request: {:?})",
            peer_id, request,
        );

        let BeaconBlocksRequest {
            head_block_root,
            start_slot,
            count,
            step,
        } = *request;

        ensure!(step != 0, EventHandlerError::SlotStepIsZero);

        let difference = count
            .checked_mul(step)
            .ok_or_else(|| EventHandlerError::SlotDifferenceOverflow { count, step })?;

        let end_slot = start_slot.checked_add(difference).ok_or_else(|| {
            EventHandlerError::EndSlotOverflow {
                start_slot,
                difference,
            }
        })?;

        Ok(Box::new(
            self.lock_networked()
                .join(self.lock_service())
                .map(move |(networked, mut service)| {
                    let beacon_blocks =
                        iter::successors(networked.get_beacon_block(head_block_root), |previous| {
                            networked.get_beacon_block(previous.parent_root)
                        })
                        .skip_while(|block| end_slot < block.slot)
                        .take_while(|block| start_slot <= block.slot)
                        .filter(|block| (block.slot - start_slot) % step == 0)
                        .cloned()
                        .collect::<Vec<_>>();

                    info!(
                        "sending BeaconBlocks response (peer_id: {}, beacon_blocks: {:?})",
                        peer_id, beacon_blocks,
                    );

                    service.swarm.send_rpc(
                        peer_id,
                        RPCEvent::Response(
                            request_id,
                            RPCErrorResponse::Success(RPCResponse::BeaconBlocks(
                                beacon_blocks.as_ssz_bytes(),
                            )),
                        ),
                    );
                }),
        ))
    }

    fn handle_recent_beacon_blocks_request(
        &self,
        peer_id: PeerId,
        request_id: RequestId,
        request: RecentBeaconBlocksRequest,
    ) -> Result<EventFuture> {
        let block_roots = request.block_roots;

        info!(
            "received RecentBeaconBlocks request (peer_id: {}, block_roots: {:?})",
            peer_id, block_roots,
        );

        Ok(Box::new(
            self.lock_networked()
                .join(self.lock_service())
                .map(move |(networked, mut service)| {
                    let beacon_blocks = block_roots
                        .into_iter()
                        .map(|root| networked.get_beacon_block(root).cloned())
                        .collect::<Vec<_>>();

                    info!(
                        "sending RecentBeaconBlocks response (peer_id: {}, beacon_blocks: {:?})",
                        peer_id, beacon_blocks,
                    );

                    service.swarm.send_rpc(
                        peer_id,
                        RPCEvent::Response(
                            request_id,
                            RPCErrorResponse::Success(RPCResponse::RecentBeaconBlocks(
                                beacon_blocks.as_ssz_bytes(),
                            )),
                        ),
                    );
                }),
        ))
    }

    fn handle_rpc_response(
        &mut self,
        peer_id: PeerId,
        response: RPCErrorResponse,
    ) -> Result<EventFuture> {
        match response {
            RPCErrorResponse::Success(RPCResponse::Hello(hello)) => {
                let remote = hello_message_into_status(hello);

                info!(
                    "received Hello response (peer_id: {}, remote: {:?})",
                    peer_id, remote,
                );

                let request_id = self.request_id()?;

                Ok(Box::new(
                    self.lock_networked().join(self.lock_service()).and_then(
                        move |(networked, service)| {
                            let local = get_and_check_status(networked.deref(), remote)?;
                            compare_status_and_request_blocks::<C>(
                                local, remote, service, peer_id, request_id,
                            );
                            Ok(())
                        },
                    ),
                ))
            }
            RPCErrorResponse::Success(RPCResponse::BeaconBlocks(bytes)) => {
                let beacon_blocks =
                    Vec::from_ssz_bytes(bytes.as_slice()).map_err(DebugAsError::new)?;

                info!(
                    "received BeaconBlocks response (peer_id: {}, beacon_blocks: {:?})",
                    peer_id, beacon_blocks,
                );

                Ok(Box::new(self.lock_networked().and_then(|mut networked| {
                    for beacon_block in beacon_blocks {
                        networked.accept_beacon_block(beacon_block)?;
                    }
                    Ok(())
                })))
            }
            RPCErrorResponse::Success(RPCResponse::RecentBeaconBlocks(response_bytes)) => {
                bail!(EventHandlerError::UnexpectedResponse {
                    peer_id,
                    response_bytes
                })
            }
            RPCErrorResponse::InvalidRequest(error_message) => {
                bail!(EventHandlerError::InvalidRequest {
                    peer_id,
                    error_message,
                })
            }
            RPCErrorResponse::ServerError(error_message) => bail!(EventHandlerError::ServerError {
                peer_id,
                error_message,
            }),
            RPCErrorResponse::Unknown(error_message) => bail!(EventHandlerError::UnknownResponse {
                peer_id,
                error_message,
            }),
        }
    }

    fn handle_peer_dialed(&mut self, peer_id: PeerId) -> Result<EventFuture> {
        info!("peer {} dialed", peer_id);

        let request_id = self.request_id()?;

        Ok(Box::new(
            self.lock_networked()
                .join(self.lock_service())
                .map(move |(networked, mut service)| {
                    let status = networked.get_status();

                    info!(
                        "sending Hello request (peer_id: {}, status: {:?})",
                        peer_id, status,
                    );

                    service.swarm.send_rpc(
                        peer_id,
                        RPCEvent::Request(
                            request_id,
                            RPCRequest::Hello(status_into_hello_message(status)),
                        ),
                    );
                }),
        ))
    }

    fn handle_pubsub_message(
        &self,
        id: String,
        source: PeerId,
        topics: Vec<TopicHash>,
        message: PubsubMessage,
    ) -> Result<EventFuture> {
        match message {
            PubsubMessage::Block(bytes) => {
                let beacon_block =
                    BeaconBlock::from_ssz_bytes(bytes.as_slice()).map_err(DebugAsError::new)?;

                info!("received beacon block as gossip: {:?}", beacon_block);

                Ok(Box::new(self.lock_networked().and_then(|mut networked| {
                    networked.accept_beacon_block(beacon_block)
                })))
            }
            PubsubMessage::Attestation(bytes) => {
                let attestation =
                    Attestation::from_ssz_bytes(bytes.as_slice()).map_err(DebugAsError::new)?;

                info!("received beacon attestation as gossip: {:?}", attestation);

                Ok(Box::new(self.lock_networked().and_then(|mut networked| {
                    networked.accept_beacon_attestation(attestation)
                })))
            }
            _ => bail!(EventHandlerError::UnsupportedGossipedObjectType {
                id,
                peer_id: source,
                topics,
                message,
            }),
        }
    }

    fn lock_networked(&self) -> impl Future<Item = Guard<N>, Error = Error> {
        self.networked.clone().lock().from_err()
    }

    fn lock_service(&self) -> impl Future<Item = Guard<Service>, Error = Error> {
        self.service.clone().lock().from_err()
    }

    fn request_id(&mut self) -> Result<usize> {
        let request_id = self.next_request_id;
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or(EventHandlerError::RequestIdsExhausted)?;
        Ok(request_id)
    }
}

// We have to implement `Future` manually because using `Stream` combinators with
// `Service` consumes it and makes it impossible to access `Service.swarm`.
//
// The implementation is roughly equivalent to:
// ```
// let handle_events = service.for_each(|libp2p_event| …);
// let publish_gossip = self.networked_receiver.0.for_each(|gossip| …);
// handle_events.select(publish_gossip)
// ```
impl<C: Config, N: Networked<C>> Future for EventHandler<C, N> {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Handle all `Libp2pEvent`s currently available from `Service`.
        loop {
            if let Some(in_progress) = &mut self.in_progress {
                try_ready!(in_progress.poll());
                self.in_progress = None;
            }
            let mut service = try_ready!(self.lock_service().poll());
            match service.poll().map_err(SyncError::new)? {
                Async::Ready(Some(libp2p_event)) => {
                    self.in_progress = Some(self.handle_libp2p_event(libp2p_event)?);
                }
                Async::Ready(None) => {
                    // See <https://github.com/sigp/lighthouse/blob/c04026d073d12a98499c9cebd6d6134fc75355a9/beacon_node/eth2-libp2p/src/service.rs#L202>.
                    unreachable!("<Service as Stream> should never end");
                }
                Async::NotReady => break,
            };
        }

        // Publish all `Gossip`s received through `networked_receiver`.
        let swarm = &mut try_ready!(self.lock_service().poll()).swarm;
        while let Some(gossip) = try_ready!(self
            .networked_receiver
            .0
            .poll()
            // Channel receivers from `futures` are supposed to never fail,
            // but `futures` 0.1 uses `()` as the `Error` type for infallible `Stream`s.
            .map_err(|()| -> Self::Error { unreachable!("UnboundedReceiver should never fail") }))
        {
            match gossip {
                Gossip::BeaconBlock(beacon_block) => swarm.publish(
                    &[Topic::new("/eth2/beacon_block/ssz".to_owned())],
                    PubsubMessage::Block(beacon_block.as_ssz_bytes()),
                ),
                Gossip::BeaconAttestation(attestation) => swarm.publish(
                    &[Topic::new("/eth2/beacon_attestation/ssz".to_owned())],
                    PubsubMessage::Attestation(attestation.as_ssz_bytes()),
                ),
            }
        }

        Ok(Async::Ready(()))
    }
}

pub fn channel<C: Config>() -> (Sender<C>, Receiver<C>) {
    let (sender, receiver) = mpsc::unbounded();
    (Sender(sender), Receiver(receiver))
}

pub fn run_network<C: Config, N: Networked<C>>(
    config: NetworkConfig,
    networked: Qutex<N>,
    networked_receiver: Receiver<C>,
) -> Result<impl Future<Item = (), Error = Error>> {
    let logger = Logger::root(StdLog.fuse(), o!());
    let service = Service::new(config, logger).map_err(SyncError::new)?;
    Ok(EventHandler {
        networked,
        networked_receiver,
        service: Qutex::new(service),
        next_request_id: 0,
        in_progress: None,
    })
}

fn hello_message_into_status(hello: HelloMessage) -> Status {
    let HelloMessage {
        fork_version,
        finalized_root,
        finalized_epoch,
        head_root,
        head_slot,
    } = hello;
    Status {
        fork_version,
        finalized_root,
        finalized_epoch: finalized_epoch.into(),
        head_root,
        head_slot: head_slot.into(),
    }
}

fn status_into_hello_message(status: Status) -> HelloMessage {
    let Status {
        fork_version,
        finalized_root,
        finalized_epoch,
        head_root,
        head_slot,
    } = status;
    HelloMessage {
        fork_version,
        finalized_root,
        finalized_epoch: finalized_epoch.into(),
        head_root,
        head_slot: head_slot.into(),
    }
}

fn get_and_check_status<C: Config, N: Networked<C>>(
    networked: &N,
    remote: Status,
) -> Result<Status> {
    let local = networked.get_status();
    ensure!(
        local.fork_version == remote.fork_version,
        EventHandlerError::ForkVersionMismatch {
            local: local.fork_version,
            remote: remote.fork_version,
        },
    );
    Ok(local)
}

fn compare_status_and_request_blocks<C: Config>(
    local: Status,
    remote: Status,
    mut service: Guard<Service>,
    peer_id: PeerId,
    request_id: RequestId,
) {
    // We currently do not check if `remote.finalized_root` is present in the local chain at
    // `remote.finalized_epoch` because there is no easy way to do it with our implementation of the
    // fork choice store.
    if (local.finalized_epoch, local.head_slot) < (remote.finalized_epoch, remote.head_slot) {
        let request = BeaconBlocksRequest {
            head_block_root: remote.head_root,
            start_slot: misc::compute_start_slot_of_epoch::<C>(remote.finalized_epoch),
            count: u64::max_value(),
            step: 1,
        };
        info!(
            "sending BeaconBlocks request (peer_id: {}, request: {:?})",
            peer_id, request,
        );
        service.swarm.send_rpc(
            peer_id,
            RPCEvent::Request(request_id, RPCRequest::BeaconBlocks(request)),
        );
    }
}
