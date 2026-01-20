//! # libp2p-alternet
//!
//!

pub mod behaviour;
pub mod control;
pub mod transport;

mod prelude {
    pub use ::libp2p::*;
    pub use Transport as _;
    pub use futures::prelude::*;
}

