# jensenlab-textmining-rs

Literature-derived entity associations from JensenLab text mining at jensenlab.org.

## Source

- **URL:** `download.jensenlab.org` (no registration required)
- **Files:** various `*_mentions.tsv.gz` — one per entity type (proteins, diseases,
  chemicals, tissues, environments, subcellular compartments, biological processes)
- **License:** CC BY 4.0
- **Update cycle:** weekly

## Usage

### Entity Mentions

Read protein mentions from a gzipped TSV file:

```rust
use std::fs::File;
use flate2::read::GzDecoder;
use jensenlab_textmining_rs::mention::EntityMentionReader;

// Open the gzipped file
let file = File::open("protein_mentions.tsv.gz")?;
let decoder = GzDecoder::new(file);

// Create reader with streaming batches
let mut reader = EntityMentionReader::new(decoder, None)?;

// Process in batches (default 8192 records per batch)
while let Some(batch_result) = reader.next() {
    let batch = batch_result?;
    println!("Read {} mentions", batch.num_rows());

    // Access columns
    let entity_ids = batch.column(0).as_any()
        .downcast_ref::<arrow::array::StringArray>()?;
    let scores = batch.column(4).as_any()
        .downcast_ref::<arrow::array::Float64Array>()?;
}
```

### Entity Associations (Pairs)

Read pairwise associations:

```rust
use jensenlab_textmining_rs::association::EntityAssociationReader;
use flate2::read::GzDecoder;

let file = File::open("protein_disease_pairs.tsv.gz")?;
let decoder = GzDecoder::new(file);
let mut reader = EntityAssociationReader::new(decoder, None)?;

for batch_result in &mut reader {
    let batch = batch_result?;
    // Process protein-disease associations
}
```

### Schema

Mention records contain:
- `entity_id`: String identifier (e.g., "ENSP00000451596")
- `entity_type`: Canonical type name (e.g., "protein", "disease")
- `entity_type_code`: Original JensenLab numeric code (e.g., 9606, -26)
- `publication_id`: PubMed or other publication identifier
- `confidence_score`: Mention confidence as f64
- `first_mention_at`: Optional timestamp of first mention

Association records contain:
- `entity_a`, `entity_b`: Entity identifiers
- `entity_a_type`, `entity_b_type`: Canonical type names
- `entity_a_type_code`, `entity_b_type_code`: Original numeric codes
- `association_type`: Relationship type (e.g., "cooccurrence")
- `publication_count`: Number of supporting publications
- `confidence_score`: Association confidence as f64

### Entity Type Taxonomy

JensenLab uses numeric type codes internally. This crate maps them to canonical names:

| Code(s) | Type Name |
|---------|-----------|
| 9606, 10090, etc. | `protein` (taxid-specific) |
| -22 | `compartment` |
| -23 | `biological_process` |
| -25 | `phenotype` |
| -26 | `disease` |
| -27 | `chemical` |
| -28 | `tissue` |
| -29 | `environment` |
| -30 | `organism` |
| -1 | `gene` |

## License

This crate is licensed under MIT OR Apache-2.0.

Parsed data is governed by the upstream CC BY 4.0 license terms from JensenLab.
