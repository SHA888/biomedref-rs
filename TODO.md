# biomedref-rs — TODO

> Open source. Apache-2.0 / MIT dual-licensed (parser code).
> Parsed data governed by each source's upstream license terms.
> Each crate versions independently.
> Format: `[ ]` open · `[x]` done · `[-]` deferred

---

## Status (v0.1, 2026-04-20)

**Scaffold only.** No crates implemented. Workspace Cargo.toml, CI, and per-crate
scaffolds deferred until a concrete consumer need arises. This TODO exists to
record scope, expected crate shape, and boundary rules before implementation
begins.

## VERSIONING

```
v0.x.x   each crate: pre-stable, breaking changes allowed in minor
v1.0.0   per crate: stable public API, MSRV policy documented
```

Each crate publishes independently. No intra-workspace dependencies are expected.

---

## SPRINT 0 — Workspace bootstrap
> Gate: first consumer request for any crate in this workspace.

- [x] S0.1 Create `github.com/SHA888/biomedref-rs` (public)
- [x] S0.2 `LICENSE-MIT` + `LICENSE-APACHE`
- [x] S0.3 `CONTRIBUTING.md` — same conventions as `multiomics-rs`
- [x] S0.4 `CODE_OF_CONDUCT.md` — Contributor Covenant v2.1
- [x] S0.5 `SECURITY.md` — data correctness bugs = security severity
- [x] S0.6 Workspace `Cargo.toml`, same template as `multiomics-rs`
- [x] S0.7 `rust-toolchain.toml` pinning latest stable
- [x] S0.8 `deny.toml` — same policy as `multiomics-rs`
- [x] S0.9 CI workflows (ci.yml, release.yml, audit.yml)
- [x] S0.10 Empty crate scaffolds (jensenlab-textmining-rs,
           exposome-explorer-rs, foodb-rs) with stub `src/lib.rs`,
           `README.md`, `CHANGELOG.md`
- [x] S0.11 `cargo check --workspace` passes on empty scaffold
- [ ] S0.12 CI green on `main`

---

## jensenlab-textmining-rs — v0.1.0
> Literature-derived entity associations from JensenLab text mining at jensenlab.org.
> Weekly updates; large open-access literature corpus processed with the Tagger tool.

**Source:** `download.jensenlab.org` (no registration required)
**Files:** various `*_mentions.tsv.gz` — one per entity type (proteins, diseases,
chemicals, tissues, environments, subcellular compartments, biological processes)
**License:** CC BY 4.0
**Update cycle:** weekly

- [x] JT1.1 TSV reader for `*_mentions.tsv.gz` (gzipped; use `flate2`)
  - `EntityMention` → RecordBatch
  - columns: entity_id (Utf8), entity_type (Utf8), publication_id (Utf8),
    confidence_score (Float64), first_mention_at (Timestamp)
- [x] JT1.2 Pairwise association reader for `*_pairs.tsv.gz` files
  - `EntityAssociation` → RecordBatch
  - columns: entity_a, entity_a_type, entity_b, entity_b_type,
    association_type, publication_count, confidence_score
- [x] JT1.3 Entity type taxonomy — JensenLab uses numeric type codes internally
           (e.g., -22 for compartments, 9606 for proteins in H. sapiens);
           emit as canonical string type names, preserve numeric code as
           metadata for traceability
- [x] JT1.4 Streaming design — text-mining files are large
           (~GB range for protein mentions); never materialize full dataset
- [x] JT1.5 CC BY 4.0 license flag in Arrow schema metadata
- [x] JT1.6 Tests against small synthetic fixtures
- [x] JT1.7 Publish `0.1.0`
- [-] JT1.8 Strict validation — reject empty required fields instead of `unwrap_or("")`
- [-] JT1.9 Separate error variant for publication count parse errors (currently reuses `InvalidScore`)
- [-] JT1.10 Schema-cloning micro-optimization (store `SchemaRef` once, clone `Arc` only)
- [-] JT1.11 Cross-reference numeric codes to taxonomy module in doc comments

---

## exposome-explorer-rs — v0.1.0  *(planned)*
> Environmental exposure biomarker associations.

**Source:** `exposome-explorer.iarc.fr` (free download, no registration)
**Files:** CSV/TSV exports of biomarkers, exposures, publications,
biospecimens, measurement methods
**License:** CC BY (IARC-hosted, open for research use)
**Update cycle:** irregular (curated releases)

- [ ] EE1.1 TSV reader for exposure-biomarker table
  - `ExposureBiomarker` → RecordBatch
  - columns: exposure_compound, biomarker_name, biospecimen,
    concentration_range, measurement_method, study_reference
- [ ] EE1.2 TSV reader for biomarker-disease association table
  - `BiomarkerAssociation` → RecordBatch
  - columns: biomarker, disease_or_outcome, effect_direction,
    evidence_level
- [ ] EE1.3 Cross-reference table — exposure compound → PubChem/ChEBI IDs
           where Exposome Explorer provides them
- [ ] EE1.4 CC BY license flag in Arrow schema metadata
- [ ] EE1.5 Tests + publish `0.1.0`

---

## foodb-rs — v0.1.0  *(planned)*
> Food composition — compounds, nutrients, cross-refs to PubChem/ChEBI/KEGG/HMDB.

**Source:** `foodb.ca/downloads` (free)
**Files:** CSV dumps — food.csv, compound.csv, content.csv, nutrient.csv,
cross-reference tables linking to PubChem / ChEBI / KEGG / HMDB
**License:** FooDB custom open terms (free for non-commercial research;
commercial use requires permission)
**Update cycle:** irregular major releases

- [ ] FD1.1 CSV reader for `food.csv`
  - `Food` → RecordBatch
  - columns: foodb_id, name, scientific_name, food_group, food_subgroup,
    description
- [ ] FD1.2 CSV reader for `compound.csv` (compounds referenced by foods)
  - `Compound` → RecordBatch
  - columns: foodb_compound_id, name, pubchem_cid, chebi_id, kegg_id,
    hmdb_id, smiles, inchi
- [ ] FD1.3 CSV reader for `content.csv` (compound concentrations per food)
  - `FoodCompound` → RecordBatch
  - columns: foodb_id, compound_id, compound_name, content_value,
    content_unit, standard_content, preparation_type
- [ ] FD1.4 CSV reader for `nutrient.csv` (nutrient concentrations per food)
  - `FoodNutrient` → RecordBatch
  - columns: foodb_id, nutrient_id, nutrient_name, content_value,
    content_unit
- [ ] FD1.5 FooDB license flag in Arrow metadata
           ("free for non-commercial research; commercial use requires permission")
- [ ] FD1.6 README documents FooDB's non-commercial clause prominently
- [ ] FD1.7 Tests + publish `0.1.0`

---

## FUTURE — Not currently planned

See [ARCHITECTURE.md](ARCHITECTURE.md) "Possible future additions" for candidates
that might join if consumer demand emerges. None are in active roadmap:

- `ctd-rs` (Comparative Toxicogenomics Database) — overlaps exposome-explorer-rs
- `envipath-rs` (enviPath) — biotransformation pathways
- `eawag-baf-rs` (Eawag-BBD) — biocatalysis/biodegradation pathways
- `usda-fooddata-rs` — partial overlap with foodb-rs

---

*Last updated: 2026-04-29. `jensenlab-textmining-rs` v0.1.0 published; review
findings deferred as maintenance items.*
