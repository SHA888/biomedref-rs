# biomedref-rs Architecture

## Purpose

`biomedref-rs` ingests freely-downloadable biomedical reference databases that fall
outside strict molecular omics, emitting Apache Arrow RecordBatches. It is a data
engineering library, not an analysis platform. Every crate has one job: read a
specific format, emit Arrow.

## What it does not do

- No clinical records (→ `clinical-rs`)
- No molecular omics reference databases (→ `multiomics-rs`)
- No license-restricted reference databases (→ `multiomics-rs-licensed`)
- No raw sequencing formats (→ `oxbow`, `noodles`)
- No statistical analysis, no model training, no inference
- No domain-specific application logic (→ consuming applications)

## Arrow as the universal contract

Same as all four sibling workspaces. Every crate emits
`arrow::record_batch::RecordBatch` via the `arrow::record_batch::RecordBatchReader`
trait. Consumers read Arrow output via PyArrow (zero-copy), Polars, DataFusion,
DuckDB, or any other Arrow-aware tool.

## Crate map

```
biomedref-rs workspace
│
├── jensenlab-textmining-rs
│     Literature-derived entity associations → Arrow
│     Reads JensenLab text-mining result files from jensenlab.org
│     (protein_mentions, disease_mentions, chemical_mentions, tissue_mentions, etc.)
│     Emits: EntityMention { entity_id, entity_type, publication_id,
│            confidence_score, first_mention_at }
│            EntityAssociation { entity_a, entity_a_type, entity_b,
│            entity_b_type, association_type, publication_count,
│            confidence_score }
│     License: CC BY 4.0
│
├── exposome-explorer-rs
│     Environmental exposure biomarker data → Arrow
│     Reads Exposome Explorer TSV / XML download files
│     Emits: ExposureBiomarker { exposure_compound, biomarker_name,
│            biospecimen, concentration_range, measurement_method,
│            study_reference }
│            BiomarkerAssociation { biomarker, disease_or_outcome,
│            effect_direction, evidence_level }
│     License: CC BY
│
└── foodb-rs
      Food composition → Arrow
      Reads FooDB TSV download (food.csv, compound.csv,
      content.csv, nutrient.csv and cross-reference tables)
      Emits: Food { foodb_id, name, scientific_name, food_group,
             food_subgroup }
             FoodCompound { foodb_id, compound_id, compound_name,
             content_value, content_unit, standard_content }
             FoodNutrient { foodb_id, nutrient_id, nutrient_name,
             content_value, content_unit }
      License: FooDB custom open terms (free for non-commercial;
               commercial use requires permission)
```

All three are Layer 1 crates in the sense used by `multiomics-rs` — they read raw
formats and emit structured Arrow records. No Layer 2 crates are currently planned
for this workspace.

## Dependency rules

```
Layer 1 crates     no dependency on each other
                   no dependency on clinical-rs
                   no dependency on multiomics-rs
                   no dependency on multiomics-rs-licensed
                   no dependency on any consuming application

Applications       depend on biomedref-rs crates (consumer)
                   biomedref-rs never imports application code
```

Each crate stands alone. A consumer using `foodb-rs` does not implicitly get
`jensenlab-textmining-rs` or `exposome-explorer-rs`.

## Format notes

*Deferred to per-crate README.md files at implementation time.* Each crate will
document its source file format, download URLs, update cycles, and schema mapping
in its own README. Keeping format notes at the per-crate level (rather than in this
workspace-wide ARCHITECTURE.md) matches the pattern used in `multiomics-rs-licensed`
and avoids drift — workspace-wide format notes get stale as sources evolve.

## Why these three sources and not more?

The scope of "biomedical reference databases outside strict molecular omics" could
easily grow to dozens of candidates. The workspace is constrained to three
starting crates on the principle that:

- Each crate must cover a genuinely useful public resource with no better existing
  Rust-native parser.
- Adjacent sources that overlap (e.g., Comparative Toxicogenomics Database overlaps
  Exposome Explorer on chemical-health relationships; SIDER-like drug-side-effect
  tables overlap SIDER which is in `multiomics-rs`) should be added only when a
  specific consumer need justifies the overlap.

This workspace is deliberately small. If it stays at three crates indefinitely,
that is a successful scope boundary, not a failure.

## Possible future additions

If concrete consumer demand emerges, candidates worth adding:

| Candidate | Source | Why it might belong here |
|---|---|---|
| `ctd-rs` | Comparative Toxicogenomics Database | Chemical-gene-disease relationships; overlaps exposome but broader |
| `envipath-rs` | enviPath | Biotransformation pathways for environmental chemicals |
| `eawag-baf-rs` | Eawag-BBD | Biocatalysis / biodegradation pathways |
| `usda-fooddata-rs` | USDA FoodData Central | Food composition; partial overlap with FooDB |

None are in the active roadmap. Any addition needs to pass the three-part test
in [README.md](README.md) "Scope boundary" section.

## Repository structure

```
biomedref-rs/
├── crates/
│   ├── jensenlab-textmining-rs/   # planned, v0.1.0
│   ├── exposome-explorer-rs/      # planned, v0.1.0
│   └── foodb-rs/                  # planned, v0.1.0
├── ARCHITECTURE.md
├── TODO.md
├── README.md
├── CONTRIBUTING.md
├── LICENSE-MIT
├── LICENSE-APACHE
└── Cargo.toml                     # planned, not yet created
```

## Status

`v0.1` — scaffold documentation only. Workspace bootstrap deferred until a
concrete consumer need for a specific crate emerges.
