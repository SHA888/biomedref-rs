//! Integration tests for `jensenlab-textmining-rs`.
//!
//! Tests use synthetic fixtures in `tests/fixtures/` directory.

use std::fs::File;
use std::io::{Cursor, Read, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use jensenlab_textmining_rs::association::{EntityAssociationReader, LICENSE_CC_BY_4_0};
use jensenlab_textmining_rs::mention::{EntityMentionReader, LICENSE_CC_BY_4_0 as MENTION_LICENSE};

/// Helper to gzip data for testing.
fn gzip_data(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

#[test]
fn test_mention_reader_with_fixture() {
    // Read the fixture file
    let mut file = File::open("tests/fixtures/protein_mentions.tsv").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // Test with uncompressed data
    let input = Cursor::new(contents.into_bytes());
    let mut reader = EntityMentionReader::new(input, Some(2)).unwrap();

    // Read first batch
    let batch1 = reader.next_batch().unwrap().unwrap();
    assert_eq!(batch1.num_rows(), 2);
    assert_eq!(batch1.num_columns(), 6);

    // Read second batch (2 more rows)
    let batch2 = reader.next_batch().unwrap().unwrap();
    assert_eq!(batch2.num_rows(), 2);

    // Read third batch (1 remaining row)
    let batch3 = reader.next_batch().unwrap().unwrap();
    assert_eq!(batch3.num_rows(), 1);

    // Now exhausted
    assert!(reader.next_batch().unwrap().is_none());
}

#[test]
fn test_mention_reader_with_gzipped_data() {
    let data = b"ENSP00000451596\t9606\t29632286\t3.77\t1451606400\n";
    let gzipped = gzip_data(data);

    let decoder = GzDecoder::new(Cursor::new(gzipped));
    let mut reader = EntityMentionReader::new(decoder, None).unwrap();

    let batch = reader.next_batch().unwrap().unwrap();
    assert_eq!(batch.num_rows(), 1);

    // Verify schema has CC BY 4.0 license
    let schema = batch.schema();
    assert_eq!(
        schema.metadata().get("license"),
        Some(&MENTION_LICENSE.to_string())
    );
}

#[test]
fn test_association_reader_with_fixture() {
    let mut file = File::open("tests/fixtures/protein_pairs.tsv").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let input = Cursor::new(contents.into_bytes());
    let mut reader = EntityAssociationReader::new(input, None).unwrap();

    let batch = reader.next_batch().unwrap().unwrap();
    assert_eq!(batch.num_rows(), 4);
    assert_eq!(batch.num_columns(), 9);

    // Verify schema
    let schema = batch.schema();
    assert_eq!(
        schema.metadata().get("license"),
        Some(&LICENSE_CC_BY_4_0.to_string())
    );
}

#[test]
fn test_entity_type_taxonomy_in_mentions() {
    let data = b"ENSP00000451596\t9606\t29632286\t3.77\n\
                  GO:0006915\t-23\t29632286\t2.45\n\
                  MESH:D000544\t-26\t29632286\t1.89\n\
                  CHEMBL25\t-27\t29632286\t4.12\n";
    let input = Cursor::new(data.to_vec());
    let mut reader = EntityMentionReader::new(input, None).unwrap();

    let batch = reader.next_batch().unwrap().unwrap();
    assert_eq!(batch.num_rows(), 4);

    // Check entity_type column
    let entity_types = batch
        .column(1)
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .unwrap();
    assert_eq!(entity_types.value(0), "protein");
    assert_eq!(entity_types.value(1), "biological_process");
    assert_eq!(entity_types.value(2), "disease");
    assert_eq!(entity_types.value(3), "chemical");

    // Check entity_type_code column is preserved
    let type_codes = batch
        .column(2)
        .as_any()
        .downcast_ref::<arrow::array::Int64Array>()
        .unwrap();
    assert_eq!(type_codes.value(0), 9606);
    assert_eq!(type_codes.value(1), -23);
    assert_eq!(type_codes.value(2), -26);
    assert_eq!(type_codes.value(3), -27);
}

#[test]
fn test_streaming_large_batches() {
    // Simulate larger data with repeated rows
    use std::fmt::Write;
    let mut data = String::new();
    for i in 0..100 {
        let _ = writeln!(
            data,
            "ENSP{:010}\t9606\t{}\t{:.2}\t1451606400",
            i,
            29_632_286 + i,
            3.0 + (f64::from(i) * 0.01)
        );
    }

    let input = Cursor::new(data.into_bytes());
    let mut reader = EntityMentionReader::new(input, Some(25)).unwrap();

    let mut total_rows = 0;
    let mut batch_count = 0;

    while let Some(batch) = reader.next_batch().unwrap() {
        total_rows += batch.num_rows();
        batch_count += 1;
    }

    assert_eq!(total_rows, 100);
    assert_eq!(batch_count, 4); // 100 rows / 25 batch_size
}

#[test]
fn test_iterator_trait() {
    let data = b"ENSP00000451596\t9606\t29632286\t3.77\n\
                  ENSP00000326845\t9606\t27150090\t2.58\n";
    let input = Cursor::new(data.to_vec());
    let reader = EntityMentionReader::new(input, None).unwrap();

    let batches: Vec<_> = reader.collect::<Result<_, _>>().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].num_rows(), 2);
}

#[test]
fn test_record_batch_reader_trait() {
    let data = b"ENSP00000451596\t9606\t29632286\t3.77\n";
    let input = Cursor::new(data.to_vec());
    let reader = EntityMentionReader::new(input, None).unwrap();

    let schema = reader.schema();
    assert_eq!(schema.fields().len(), 6);
}

#[test]
fn test_disease_mentions_fixture() {
    let mut file = File::open("tests/fixtures/disease_mentions.tsv").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let input = Cursor::new(contents.into_bytes());
    let mut reader = EntityMentionReader::new(input, None).unwrap();

    let batch = reader.next_batch().unwrap().unwrap();
    assert_eq!(batch.num_rows(), 3);

    // All should be disease type
    let entity_types = batch
        .column(1)
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .unwrap();
    for i in 0..3 {
        assert_eq!(entity_types.value(i), "disease");
    }
}

#[test]
#[allow(clippy::similar_names)] // entity_a_type/_b_type are semantically correct for pairs
fn test_protein_to_disease_association() {
    let data = b"ENSP00000451596\t9606\tMESH:D000544\t-26\tcooccurrence\t5\t7.55\n";
    let input = Cursor::new(data.to_vec());
    let mut reader = EntityAssociationReader::new(input, None).unwrap();

    let batch = reader.next_batch().unwrap().unwrap();
    assert_eq!(batch.num_rows(), 1);

    // Entity A is protein
    let entity_a_type = batch
        .column(1)
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .unwrap();
    assert_eq!(entity_a_type.value(0), "protein");

    // Entity B is disease
    let entity_b_type = batch
        .column(4)
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .unwrap();
    assert_eq!(entity_b_type.value(0), "disease");
}
