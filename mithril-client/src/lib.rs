#![warn(missing_docs)]

//! Define everything necessary to list, download, and validate snapshots from a
//! [Mithril Aggregator](https://mithril.network/rust-doc/mithril_aggregator/index.html).
//!
//! To query an aggregator for snapshots & certificate use the [services::SnapshotService].
//!

pub mod aggregator_client;
pub mod client;
pub mod dependencies;
mod entities;
mod message_adapters;
pub mod services;
pub mod utils;

pub use entities::*;
pub use message_adapters::{FromCertificateMessageAdapter, FromSnapshotMessageAdapter};

/// `mithril-common` re-exports
pub mod common {
    pub use mithril_common::{
        certificate_chain::CertificateVerifier,
        crypto_helper::{ProtocolGenesisVerificationKey, ProtocolGenesisVerifier},
        entities::{Beacon, CompressionAlgorithm, Epoch},
        messages::{
            MithrilStakeDistributionListMessage, SnapshotListItemMessage, SnapshotListMessage,
            SnapshotMessage,
        },
        StdError, StdResult,
    };
}
