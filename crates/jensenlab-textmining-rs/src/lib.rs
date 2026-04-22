//! Literature-derived entity associations from `JensenLab` text mining.
//!
//! Reads `JensenLab` text-mining result files from jensenlab.org and emits
//! Apache Arrow [`RecordBatch`] records.
//!
//! Source: `download.jensenlab.org` (no registration required)
//! License: CC BY 4.0

#![warn(missing_docs)]

// TODO: Implement EntityMention and EntityAssociation readers
