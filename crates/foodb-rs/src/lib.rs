//! Food composition — compounds, nutrients, cross-refs to PubChem/ChEBI/KEGG/HMDB.
//!
//! Reads `FooDB` CSV download files and emits Apache Arrow [`RecordBatch`] records.
//!
//! Source: `foodb.ca/downloads` (free)
//! License: `FooDB` custom open terms (free for non-commercial research;
//!          commercial use requires permission)

#![warn(missing_docs)]

// TODO: Implement Food, Compound, FoodCompound, FoodNutrient readers
