//! Environmental exposure biomarker associations.
//!
//! Reads `Exposome Explorer` TSV / CSV download files and emits
//! Apache Arrow [`RecordBatch`] records.
//!
//! Source: `exposome-explorer.iarc.fr` (free download, no registration)
//! License: CC BY (IARC-hosted, open for research use)

#![warn(missing_docs)]

// TODO: Implement ExposureBiomarker and BiomarkerAssociation readers
