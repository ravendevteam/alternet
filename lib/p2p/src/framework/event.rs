use super::*;

pub use Event::Dcutr;
pub use Event::Gossipsub;
pub use Event::Identify;
pub use Event::Kad;
pub use Event::Mdns;
pub use Event::Ping;
pub use Event::Relay;

pub use libp2p::ping::Event as PingEvent;
pub use libp2p::gossipsub::Event::GossipsubNotSupported;
pub use libp2p::dcutr::Event as DcutrEvent;
pub use libp2p::swarm::SwarmEvent::Behaviour;
pub use libp2p::swarm::SwarmEvent::ConnectionClosed;
pub use libp2p::swarm::SwarmEvent::ConnectionEstablished;
pub use libp2p::swarm::SwarmEvent::Dialing;
pub use libp2p::swarm::SwarmEvent::ExpiredListenAddr;
pub use libp2p::swarm::SwarmEvent::ExternalAddrConfirmed;
pub use libp2p::swarm::SwarmEvent::ExternalAddrExpired;
pub use libp2p::swarm::SwarmEvent::IncomingConnection;
pub use libp2p::swarm::SwarmEvent::IncomingConnectionError;
pub use libp2p::swarm::SwarmEvent::ListenerClosed;
pub use libp2p::swarm::SwarmEvent::ListenerError;
pub use libp2p::swarm::SwarmEvent::NewExternalAddrCandidate;
pub use libp2p::swarm::SwarmEvent::NewExternalAddrOfPeer;
pub use libp2p::swarm::SwarmEvent::NewListenAddr;
pub use libp2p::swarm::SwarmEvent::OutgoingConnectionError;
pub use libp2p::kad::Event as KadEvent;
pub use libp2p::mdns::Event as MdnsEvent;
pub use libp2p::relay::Event as RelayEvent;
pub use libp2p::request_response::Event as ReqResEvent;
pub use libp2p::identify::Event as IdentifyEvent;
pub use libp2p::gossipsub::Event as GossipsubEvent;

macro_rules! alias {
    (
        $vis:vis
        $(
            $path:path => $prefix:ident {
                $($var:ident)*
            }
        )*
    ) => {
        $(
            paste::paste! {
                $(
                    $vis use $path::$var as [<$prefix $var>];
                )*
            }
        )*
    };
}

alias!(
    pub

    libp2p::kad::Event => Kad {
        InboundRequest
        OutboundQueryProgressed
        RoutingUpdated
        UnroutablePeer
        RoutablePeer
        PendingRoutablePeer
        ModeChanged
    }

    libp2p::mdns::Event => Mdns {
        Discovered
        Expired
    }

    libp2p::relay::Event => Relay {
        ReservationClosed
        ReservationReqAccepted
        ReservationReqDenied
        ReservationTimedOut
        CircuitClosed
        CircuitReqAccepted
        CircuitReqDenied
    }

    libp2p::request_response::Event => ReqRes {
        Message
        OutboundFailure
        InboundFailure
        ResponseSent
    }

    libp2p::identify::Event => Identify {
        Received
        Sent
        Pushed
        Error      
    }

    libp2p::gossipsub::Event => Gossipsub {
        Message
        Subscribed
        Unsubscribed
        SlowPeer
    }
);