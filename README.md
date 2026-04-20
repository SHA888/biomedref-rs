<div align="center">

# biomedref-rs

**Rust Arrow parsers for biomedical reference databases outside strict molecular omics.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE-APACHE)

[Architecture](ARCHITECTURE.md) · [Roadmap](TODO.md)

</div>

---

## What is this?

`biomedref-rs` is a Cargo workspace of Rust parsers for biomedical reference databases
that fall outside strict molecular omics. These are public, freely downloadable
sources that provide context most useful *alongside* molecular data: literature-mining
associations, environmental exposure biomarkers, food composition. Every crate reads
a specific database format and emits Apache Arrow RecordBatches.

| Crate | Source | Domain | License | Status |
|---|---|---|---|---|
| [`jensenlab-textmining-rs`](crates/jensenlab-textmining-rs) | JensenLab text-mining results | Literature → entity associations | CC BY | 📋 Planned |
| [`exposome-explorer-rs`](crates/exposome-explorer-rs) | Exposome Explorer | Environmental exposure → biomarker | CC BY | 📋 Planned |
| [`foodb-rs`](crates/foodb-rs) | FooDB | Food composition | custom open | 📋 Planned |

All three crates are **planned, not yet implemented** as of v0.1 of this document
(2026-04-20). The workspace currently exists at the scaffold level (README,
ARCHITECTURE, TODO) pending a concrete consumer need.

## Why a separate workspace?

A single "biomedical reference" workspace holding everything from UniProt to FooDB
would have a scope so broad it's effectively undefined. The four-workspace split
draws two orthogonal boundaries:

- **License / access** splits `multiomics-rs` (freely downloadable, no signed
  agreement) from `multiomics-rs-licensed` (signed agreement or paid tier required).
- **Subject matter** splits `multiomics-rs` (strict molecular omics — genes,
  proteins, small molecules, pathways, interactions) from `biomedref-rs`
  (biomedical reference data that *contextualizes* molecular omics but isn't itself
  molecular: text-mining, environmental exposure, food composition).

Keeping `biomedref-rs` separate means `multiomics-rs` has a tighter scope claim
("molecular omics databases in Rust") and `biomedref-rs` has its own — smaller but
defensible — identity.

This workspace is **small on purpose**. Three crates is fine. If a single workspace
will never have more than five or six crates in it, that's a successful boundary:
it's doing its job by preventing its siblings from absorbing these sources.

## Sibling workspaces

```
clinical-rs              clinical records (MIMIC, ICD codes, task windowing)
                         github.com/SHA888/clinical-rs

multiomics-rs            molecular reference databases, fully open
                         github.com/SHA888/multiomics-rs

multiomics-rs-licensed   molecular references requiring license agreements
                         github.com/SHA888/multiomics-rs-licensed

biomedref-rs             this workspace — biomedical references outside strict
                         molecular omics (literature-mining, environmental
                         exposure, food composition)
                         github.com/SHA888/biomedref-rs
```

None of the four workspaces depends on any other. All four emit Apache Arrow
RecordBatches as the common contract. Consumers declare dependencies on whichever
workspaces they need.

## Scope boundary — test for inclusion

A candidate crate belongs in `biomedref-rs` if all three hold:

1. **Biomedical reference data** — not experimental results, not clinical records
2. **Not strictly molecular omics** — text-mining, exposure, food, environment, etc.
3. **Freely downloadable** — no signed license agreement, no paid tier required for
   the data used by the parser (restrictive licensing would move it to a hypothetical
   `biomedref-rs-licensed`, which is not planned and does not currently exist)

Borderline cases:
- **SIDER** (drug side effects) → goes to `multiomics-rs` because pharmacology/drug
  metadata is molecular-adjacent (same shelf as DGIdb, DrugBank).
- **DisGeNet** (gene-disease associations) → would go to `multiomics-rs` except that
  2024+ licensing moved it behind academic application, so it's in
  `multiomics-rs-licensed`.
- **JensenLab text-mining** → this workspace, because the primary product is
  protein–publication and drug–publication associations derived from literature
  mining, not a molecular reference itself.

## Quick start — expected usage pattern

When crates ship, the typical consumer workflow will be:

```toml
# Cargo.toml
[dependencies]
jensenlab-textmining-rs = "0.1"   # when published
```

```rust
use jensenlab_textmining_rs::TextminingReader;

let reader = TextminingReader::open("protein_mentions.tsv.gz")?;
let batches = reader.entity_publication_batches()?;
```

## Status

`v0.1` — Scaffold-level documentation only. No crates implemented. Workspace
bootstrap (Cargo.toml, CI, crate stubs) deferred until concrete consumer need for a
specific crate.

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE), at your
option. Upstream data sources carry their own terms; this workspace surfaces
license information in Arrow schema metadata for each emitted RecordBatch.

## Citation

```bibtex
@software{biomedref_rs,
  author  = {Kresna Sucandra},
  title   = {biomedref-rs: Rust Arrow parsers for biomedical reference databases outside strict molecular omics},
  url     = {https://github.com/SHA888/biomedref-rs},
  license = {MIT OR Apache-2.0},
}
```
