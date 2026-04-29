//! Entity mention reader for `JensenLab` text mining files.
//!
//! Parses `*_mentions.tsv.gz` files and emits Arrow `RecordBatch` records.
//! Files are processed in streaming fashion to handle large datasets (GB scale).

use std::io::Read;

use arrow::array::{ArrayRef, Float64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use arrow::error::ArrowError;
use arrow::record_batch::{RecordBatch, RecordBatchReader};
use chrono::{DateTime, Utc};

use crate::error::{Error, Result};
use crate::taxonomy::EntityType;

/// CC BY 4.0 license identifier for schema metadata.
pub const LICENSE_CC_BY_4_0: &str = "CC-BY-4.0";

/// Default batch size for streaming reads.
pub const DEFAULT_BATCH_SIZE: usize = 8192;

/// Schema for entity mention records.
#[must_use]
pub fn mention_schema() -> SchemaRef {
    let fields = vec![
        Field::new("entity_id", DataType::Utf8, false),
        Field::new("entity_type", DataType::Utf8, false),
        Field::new("entity_type_code", DataType::Int64, true),
        Field::new("publication_id", DataType::Utf8, false),
        Field::new("confidence_score", DataType::Float64, false),
        Field::new(
            "first_mention_at",
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            true,
        ),
    ];

    let metadata = [("license".to_string(), LICENSE_CC_BY_4_0.to_string())]
        .into_iter()
        .collect();

    SchemaRef::new(Schema::new_with_metadata(fields, metadata))
}

/// A single entity mention record.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMention {
    /// Entity identifier (e.g., "ENSP00000451596" for proteins).
    pub entity_id: String,
    /// Canonical entity type name.
    pub entity_type: EntityType,
    /// Original numeric type code from `JensenLab`.
    pub entity_type_code: i64,
    /// Publication identifier where the mention occurred.
    pub publication_id: String,
    /// Confidence score (0.0 to 1.0+).
    pub confidence_score: f64,
    /// Timestamp of first mention (if available).
    pub first_mention_at: Option<DateTime<Utc>>,
}

impl EntityMention {
    /// Parse a single row from the TSV file.
    ///
    /// Expected TSV format (tab-separated):
    /// `entity_id<TAB>entity_type_code<TAB>publication_id<TAB>confidence_score<TAB>timestamp`
    ///
    /// # Errors
    ///
    /// Returns an error if the row format is invalid or data cannot be parsed.
    pub fn from_row(row: &csv::StringRecord) -> crate::error::Result<Self> {
        if row.len() < 4 {
            return Err(Error::Csv(csv::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unequal lengths: expected {}, got {}", 4, row.len()),
            ))));
        }

        let entity_id = row.get(0).unwrap_or("").to_string();
        let entity_type_code: i64 = row
            .get(1)
            .unwrap_or("0")
            .parse()
            .map_err(|_| Error::InvalidEntityType(0))?;
        let entity_type = EntityType::from_code(entity_type_code);
        let publication_id = row.get(2).unwrap_or("").to_string();
        let confidence_score: f64 = row
            .get(3)
            .unwrap_or("0.0")
            .parse()
            .map_err(|e| Error::InvalidScore(format!("{}: {:?}", e, row.get(3))))?;

        let first_mention_at = if row.len() > 4 {
            row.get(4)
                .filter(|s| !s.is_empty())
                .and_then(|ts| ts.parse::<i64>().ok())
                .and_then(|unix_ts| DateTime::from_timestamp(unix_ts, 0).map(|dt| dt.to_utc()))
        } else {
            None
        };

        Ok(EntityMention {
            entity_id,
            entity_type,
            entity_type_code,
            publication_id,
            confidence_score,
            first_mention_at,
        })
    }
}

/// Streaming reader for entity mention TSV files.
///
/// This reader processes files in batches to avoid materializing
/// the entire dataset in memory (critical for GB-scale protein mention files).
pub struct EntityMentionReader<R: Read> {
    reader: csv::Reader<R>,
    schema: SchemaRef,
    batch_size: usize,
}

impl<R: Read> EntityMentionReader<R> {
    /// Create a new reader from a TSV input.
    ///
    /// # Arguments
    ///
    /// * `input` - A readable input stream (typically from a `.gz` file).
    /// * `batch_size` - Number of records per batch (default: 8192).
    ///
    /// # Errors
    ///
    /// Returns an error if the input cannot be read.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use flate2::read::GzDecoder;
    /// use jensenlab_textmining_rs::mention::EntityMentionReader;
    ///
    /// let file = File::open("protein_mentions.tsv.gz").unwrap();
    /// let decoder = GzDecoder::new(file);
    /// let reader = EntityMentionReader::new(decoder, None).unwrap();
    /// ```
    pub fn new(input: R, batch_size: Option<usize>) -> Result<Self> {
        let batch_size = batch_size.unwrap_or(DEFAULT_BATCH_SIZE);

        let reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(input);

        Ok(Self {
            reader,
            schema: mention_schema(),
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
        let mut records: Vec<EntityMention> = Vec::with_capacity(self.batch_size);

        for result in self.reader.records().take(self.batch_size) {
            let row = result.map_err(Error::Csv)?;
            records.push(EntityMention::from_row(&row)?);
        }

        if records.is_empty() {
            return Ok(None);
        }

        Ok(Some(build_batch(&records, &self.schema)?))
    }
}

/// Build a `RecordBatch` from entity mention records.
fn build_batch(records: &[EntityMention], schema: &SchemaRef) -> crate::error::Result<RecordBatch> {
    let entity_ids: Vec<&str> = records.iter().map(|r| r.entity_id.as_str()).collect();
    let entity_types: Vec<&str> = records.iter().map(|r| r.entity_type.as_str()).collect();
    let entity_type_codes: Vec<i64> = records.iter().map(|r| r.entity_type_code).collect();
    let publication_ids: Vec<&str> = records.iter().map(|r| r.publication_id.as_str()).collect();
    let confidence_scores: Vec<f64> = records.iter().map(|r| r.confidence_score).collect();
    let first_mention_ats: Vec<Option<i64>> = records
        .iter()
        .map(|r| {
            r.first_mention_at
                .map(|dt| dt.timestamp_nanos_opt().unwrap_or(0))
        })
        .collect();

    // Build timestamp array with proper timezone handling
    let timestamps: arrow::array::PrimitiveArray<arrow::datatypes::TimestampNanosecondType> =
        first_mention_ats.iter().copied().collect();

    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(entity_ids)) as ArrayRef,
        Arc::new(StringArray::from(entity_types)) as ArrayRef,
        Arc::new(arrow::array::Int64Array::from(entity_type_codes)) as ArrayRef,
        Arc::new(StringArray::from(publication_ids)) as ArrayRef,
        Arc::new(Float64Array::from(confidence_scores)) as ArrayRef,
        Arc::new(timestamps) as ArrayRef,
    ];

    Ok(RecordBatch::try_new(schema.clone(), columns)?)
}

use std::sync::Arc;

impl<R: Read> Iterator for EntityMentionReader<R> {
    type Item = std::result::Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_batch() {
            Ok(Some(batch)) => Some(Ok(batch)),
            Ok(None) => None,
            Err(e) => Some(Err(ArrowError::ExternalError(Box::new(e)))),
        }
    }
}

impl<R: Read> RecordBatchReader for EntityMentionReader<R> {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn sample_tsv() -> Vec<u8> {
        b"ENSP00000451596\t9606\t29632286\t3.77\t1451606400
ENSP00000326845\t9606\t27150090\t2.58\t1461974400
"
        .to_vec()
    }

    #[test]
    fn test_entity_mention_from_row() {
        let data = sample_tsv();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(data.as_slice());

        let row = reader.records().next().unwrap().unwrap();
        let mention = EntityMention::from_row(&row).unwrap();

        assert_eq!(mention.entity_id, "ENSP00000451596");
        assert_eq!(mention.entity_type, EntityType::Protein);
        assert_eq!(mention.entity_type_code, 9606);
        assert_eq!(mention.publication_id, "29632286");
        assert!(mention.confidence_score > 3.0);
    }

    #[test]
    fn test_entity_mention_reader() {
        let data = sample_tsv();
        let input = Cursor::new(data);
        let mut reader = EntityMentionReader::new(input, Some(10)).unwrap();

        let batch = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 6);

        // Check schema has license metadata
        let schema = batch.schema();
        assert_eq!(
            schema.metadata().get("license"),
            Some(&"CC-BY-4.0".to_string())
        );
    }

    #[test]
    fn test_mention_schema() {
        let schema = mention_schema();
        assert_eq!(schema.fields().len(), 6);
        assert_eq!(schema.field(0).name(), "entity_id");
        assert_eq!(schema.field(1).name(), "entity_type");
        assert_eq!(schema.field(2).name(), "entity_type_code");
        assert_eq!(schema.field(3).name(), "publication_id");
        assert_eq!(schema.field(4).name(), "confidence_score");
        assert_eq!(schema.field(5).name(), "first_mention_at");

        // Check metadata
        assert_eq!(
            schema.metadata().get("license"),
            Some(&"CC-BY-4.0".to_string())
        );
    }
}
