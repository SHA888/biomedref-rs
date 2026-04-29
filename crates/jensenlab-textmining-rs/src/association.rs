//! Pairwise association reader for `JensenLab` text mining files.
//!
//! Parses `*_pairs.tsv.gz` files containing entity-entity associations
//! and emits Arrow `RecordBatch` records.

use std::io::Read;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::{RecordBatch, RecordBatchReader};

use crate::error::{Error, Result};
use crate::taxonomy::EntityType;

/// CC BY 4.0 license identifier for schema metadata.
pub const LICENSE_CC_BY_4_0: &str = "CC-BY-4.0";

/// Default batch size for streaming reads.
pub const DEFAULT_BATCH_SIZE: usize = 8192;

/// Schema for entity association records.
#[must_use]
pub fn association_schema() -> SchemaRef {
    let fields = vec![
        Field::new("entity_a", DataType::Utf8, false),
        Field::new("entity_a_type", DataType::Utf8, false),
        Field::new("entity_a_type_code", DataType::Int64, true),
        Field::new("entity_b", DataType::Utf8, false),
        Field::new("entity_b_type", DataType::Utf8, false),
        Field::new("entity_b_type_code", DataType::Int64, true),
        Field::new("association_type", DataType::Utf8, false),
        Field::new("publication_count", DataType::Int64, false),
        Field::new("confidence_score", DataType::Float64, false),
    ];

    let metadata = [("license".to_string(), LICENSE_CC_BY_4_0.to_string())]
        .into_iter()
        .collect();

    SchemaRef::new(Schema::new_with_metadata(fields, metadata))
}

/// A single entity association (pairwise relationship) record.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityAssociation {
    /// First entity identifier.
    pub entity_a: String,
    /// Canonical type name for entity A.
    pub entity_a_type: EntityType,
    /// Original numeric type code for entity A.
    pub entity_a_type_code: i64,
    /// Second entity identifier.
    pub entity_b: String,
    /// Canonical type name for entity B.
    pub entity_b_type: EntityType,
    /// Original numeric type code for entity B.
    pub entity_b_type_code: i64,
    /// Type of association (e.g., "cooccurrence", "interaction").
    pub association_type: String,
    /// Number of publications supporting this association.
    pub publication_count: i64,
    /// Confidence score for this association.
    pub confidence_score: f64,
}

impl EntityAssociation {
    /// Parse a single row from the pairs TSV file.
    ///
    /// Expected TSV format:
    /// `entity_a<TAB>type_a<TAB>entity_b<TAB>type_b<TAB>assoc_type<TAB>pub_count<TAB>confidence`
    ///
    /// # Errors
    ///
    /// Returns an error if the row format is invalid or data cannot be parsed.
    #[allow(clippy::similar_names)] // entity_a_type/_code naming is semantically correct for pairs
    pub fn from_row(row: &csv::StringRecord) -> crate::error::Result<Self> {
        if row.len() < 7 {
            return Err(Error::Csv(csv::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unequal lengths: expected {}, got {}", 7, row.len()),
            ))));
        }

        let entity_a = row.get(0).unwrap_or("").to_string();
        let entity_a_type_code: i64 = row
            .get(1)
            .unwrap_or("0")
            .parse()
            .map_err(|_| Error::InvalidEntityType(0))?;
        let entity_a_type = EntityType::from_code(entity_a_type_code);

        let entity_b = row.get(2).unwrap_or("").to_string();
        let entity_b_type_code: i64 = row
            .get(3)
            .unwrap_or("0")
            .parse()
            .map_err(|_| Error::InvalidEntityType(0))?;
        let entity_b_type = EntityType::from_code(entity_b_type_code);

        let association_type = row.get(4).unwrap_or("unknown").to_string();

        let publication_count: i64 = row
            .get(5)
            .unwrap_or("0")
            .parse()
            .map_err(|e| Error::InvalidScore(format!("pub count: {}: {:?}", e, row.get(5))))?;

        let confidence_score: f64 = row
            .get(6)
            .unwrap_or("0.0")
            .parse()
            .map_err(|e| Error::InvalidScore(format!("confidence: {}: {:?}", e, row.get(6))))?;

        Ok(EntityAssociation {
            entity_a,
            entity_a_type,
            entity_a_type_code,
            entity_b,
            entity_b_type,
            entity_b_type_code,
            association_type,
            publication_count,
            confidence_score,
        })
    }
}

/// Streaming reader for entity association (pairs) TSV files.
///
/// This reader processes files in batches to handle large pairwise datasets.
pub struct EntityAssociationReader<R: Read> {
    reader: csv::Reader<R>,
    schema: SchemaRef,
    batch_size: usize,
}

impl<R: Read> EntityAssociationReader<R> {
    /// Create a new reader from a TSV input.
    ///
    /// # Arguments
    ///
    /// * `input` - A readable input stream.
    /// * `batch_size` - Number of records per batch (default: 8192).
    ///
    /// # Errors
    ///
    /// Returns an error if the input cannot be read.
    pub fn new(input: R, batch_size: Option<usize>) -> Result<Self> {
        let batch_size = batch_size.filter(|&s| s > 0).unwrap_or(DEFAULT_BATCH_SIZE);

        let reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(input);

        Ok(Self {
            reader,
            schema: association_schema(),
            batch_size,
        })
    }

    /// Get the schema for this reader.
    pub fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    /// Read the next batch of records.
    ///
    /// Returns `Ok(None)` when the input is exhausted.
    ///
    /// # Errors
    ///
    /// Returns an error if CSV parsing fails or a row cannot be parsed.
    pub fn next_batch(&mut self) -> crate::error::Result<Option<RecordBatch>> {
        let mut records: Vec<EntityAssociation> = Vec::with_capacity(self.batch_size);

        for result in self.reader.records().take(self.batch_size) {
            let row = result.map_err(Error::Csv)?;
            records.push(EntityAssociation::from_row(&row)?);
        }

        if records.is_empty() {
            return Ok(None);
        }

        Ok(Some(build_batch(&records, &self.schema)?))
    }
}

/// Build a `RecordBatch` from entity association records.
#[allow(clippy::similar_names)] // entity_a/entity_b naming is semantically correct for pairs
fn build_batch(
    records: &[EntityAssociation],
    schema: &SchemaRef,
) -> crate::error::Result<RecordBatch> {
    let entity_a: Vec<&str> = records.iter().map(|r| r.entity_a.as_str()).collect();
    let entity_a_type: Vec<&str> = records.iter().map(|r| r.entity_a_type.as_str()).collect();
    let entity_a_type_code: Vec<i64> = records.iter().map(|r| r.entity_a_type_code).collect();
    let entity_b: Vec<&str> = records.iter().map(|r| r.entity_b.as_str()).collect();
    let entity_b_type: Vec<&str> = records.iter().map(|r| r.entity_b_type.as_str()).collect();
    let entity_b_type_code: Vec<i64> = records.iter().map(|r| r.entity_b_type_code).collect();
    let association_type: Vec<&str> = records
        .iter()
        .map(|r| r.association_type.as_str())
        .collect();
    let publication_count: Vec<i64> = records.iter().map(|r| r.publication_count).collect();
    let confidence_score: Vec<f64> = records.iter().map(|r| r.confidence_score).collect();

    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(entity_a)) as ArrayRef,
        Arc::new(StringArray::from(entity_a_type)) as ArrayRef,
        Arc::new(Int64Array::from(entity_a_type_code)) as ArrayRef,
        Arc::new(StringArray::from(entity_b)) as ArrayRef,
        Arc::new(StringArray::from(entity_b_type)) as ArrayRef,
        Arc::new(Int64Array::from(entity_b_type_code)) as ArrayRef,
        Arc::new(StringArray::from(association_type)) as ArrayRef,
        Arc::new(Int64Array::from(publication_count)) as ArrayRef,
        Arc::new(Float64Array::from(confidence_score)) as ArrayRef,
    ];

    Ok(RecordBatch::try_new(schema.clone(), columns)?)
}

impl<R: Read> Iterator for EntityAssociationReader<R> {
    type Item = std::result::Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_batch() {
            Ok(Some(batch)) => Some(Ok(batch)),
            Ok(None) => None,
            Err(e) => Some(Err(ArrowError::ExternalError(Box::new(e)))),
        }
    }
}

impl<R: Read> RecordBatchReader for EntityAssociationReader<R> {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn sample_pairs_tsv() -> Vec<u8> {
        b"ENSP00000451596\t9606\t29632286\t-26\tcooccurrence\t5\t7.55
ENSP00000326845\t9606\tMESH:D123456\t-26\tcooccurrence\t3\t4.22
"
        .to_vec()
    }

    #[test]
    fn test_entity_association_from_row() {
        let data = sample_pairs_tsv();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(data.as_slice());

        let row = reader.records().next().unwrap().unwrap();
        let assoc = EntityAssociation::from_row(&row).unwrap();

        assert_eq!(assoc.entity_a, "ENSP00000451596");
        assert_eq!(assoc.entity_a_type, EntityType::Protein);
        assert_eq!(assoc.entity_a_type_code, 9606);
        assert_eq!(assoc.entity_b, "29632286");
        assert_eq!(assoc.entity_b_type, EntityType::Disease);
        assert_eq!(assoc.association_type, "cooccurrence");
        assert_eq!(assoc.publication_count, 5);
        assert!(assoc.confidence_score > 7.0);
    }

    #[test]
    fn test_entity_association_reader() {
        let data = sample_pairs_tsv();
        let input = Cursor::new(data);
        let mut reader = EntityAssociationReader::new(input, Some(10)).unwrap();

        let batch = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 9);

        // Check schema has license metadata
        let schema = batch.schema();
        assert_eq!(
            schema.metadata().get("license"),
            Some(&"CC-BY-4.0".to_string())
        );
    }

    #[test]
    fn test_association_schema() {
        let schema = association_schema();
        assert_eq!(schema.fields().len(), 9);
        assert_eq!(schema.field(0).name(), "entity_a");
        assert_eq!(schema.field(1).name(), "entity_a_type");
        assert_eq!(schema.field(2).name(), "entity_a_type_code");
        assert_eq!(schema.field(3).name(), "entity_b");
        assert_eq!(schema.field(4).name(), "entity_b_type");
        assert_eq!(schema.field(5).name(), "entity_b_type_code");
        assert_eq!(schema.field(6).name(), "association_type");
        assert_eq!(schema.field(7).name(), "publication_count");
        assert_eq!(schema.field(8).name(), "confidence_score");

        // Check metadata
        assert_eq!(
            schema.metadata().get("license"),
            Some(&"CC-BY-4.0".to_string())
        );
    }
}
