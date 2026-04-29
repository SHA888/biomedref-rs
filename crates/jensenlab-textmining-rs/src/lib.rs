//! Literature-derived entity associations from `JensenLab` text mining.
//!
//! Reads `JensenLab` text-mining result files from jensenlab.org and emits
//! Apache Arrow [`RecordBatch`] records.
//!
//! This crate provides streaming readers for:
//! - `*_mentions.tsv.gz` files (entity mentions)
//! - `*_pairs.tsv.gz` files (pairwise entity associations)
//!
//! # Example
//!
//! ```no_run
//! use std::fs::File;
//! use flate2::read::GzDecoder;
//! use jensenlab_textmining_rs::mention::EntityMentionReader;
//!
//! // Read protein mentions
//! let file = File::open("protein_mentions.tsv.gz").unwrap();
//! let decoder = GzDecoder::new(file);
//! let mut reader = EntityMentionReader::new(decoder, None).unwrap();
//!
//! // Stream through batches
//! for batch_result in &mut reader {
//!     let batch = batch_result.unwrap();
//!     println!("Read {} mentions", batch.num_rows());
//! }
//! ```
//!
//! # Data License
//!
//! Source: `download.jensenlab.org` (no registration required)
//! License: CC BY 4.0

#![warn(missing_docs)]

pub mod association;
pub mod error;
pub mod mention;
pub mod taxonomy;

pub use association::{EntityAssociation, EntityAssociationReader};
pub use error::{Error, Result};
pub use mention::{EntityMention, EntityMentionReader};
pub use taxonomy::EntityType;
