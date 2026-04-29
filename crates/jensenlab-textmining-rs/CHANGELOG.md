# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-29

### Added

- `EntityMentionReader` for streaming TSV mentions files (`*_mentions.tsv.gz`)
- `EntityAssociationReader` for pairwise association files (`*_pairs.tsv.gz`)
- Entity type taxonomy mapping numeric codes to canonical names
- Arrow `RecordBatch` output with CC BY 4.0 license metadata
- Streaming batch processing (default 8192 records/batch) for large files
- Integration tests with synthetic fixtures
