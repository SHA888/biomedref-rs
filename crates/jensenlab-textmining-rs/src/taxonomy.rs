//! Entity type taxonomy for `JensenLab` text mining.
//!
//! `JensenLab` uses numeric type codes internally. This module maps those codes
//! to canonical string type names while preserving the numeric code as metadata
//! for traceability.

/// Entity type classification from `JensenLab` text mining.
///
/// Numeric codes are preserved for traceability but canonical string names
/// are used for the public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EntityType {
    /// Protein mentions (e.g., code 9606 for *H. sapiens* proteins).
    Protein,
    /// Disease mentions.
    Disease,
    /// Chemical/drug mentions.
    Chemical,
    /// Tissue mentions.
    Tissue,
    /// Environment mentions.
    Environment,
    /// Subcellular compartment mentions (code -22).
    Compartment,
    /// Biological process mentions.
    BiologicalProcess,
    /// Phenotype mentions.
    Phenotype,
    /// Gene mentions.
    Gene,
    /// Organism mentions.
    Organism,
    /// Unknown or unmapped entity type.
    #[default]
    Unknown,
}

impl EntityType {
    /// Convert a `JensenLab` numeric type code to an `EntityType`.
    ///
    /// # Examples
    ///
    /// ```
    /// use jensenlab_textmining_rs::taxonomy::EntityType;
    ///
    /// assert_eq!(EntityType::from_code(9606), EntityType::Protein);
    /// assert_eq!(EntityType::from_code(-22), EntityType::Compartment);
    /// ```
    #[must_use]
    #[allow(clippy::match_same_arms)] // Multiple taxids intentionally map to Protein
    pub fn from_code(code: i64) -> Self {
        match code {
            // Human proteins (9606 = Homo sapiens taxid)
            9606 => EntityType::Protein,
            // Common protein taxids (mouse, rat, fly, worm, zebrafish, yeast)
            10090 | 10116 | 7227 | 6239 | 7955 | 4932 => EntityType::Protein,
            // Subcellular compartments use negative code -22
            -22 => EntityType::Compartment,
            // Disease
            -26 => EntityType::Disease,
            // Chemical
            -27 => EntityType::Chemical,
            // Tissue
            -28 => EntityType::Tissue,
            // Environment
            -29 => EntityType::Environment,
            // Biological process
            -23 => EntityType::BiologicalProcess,
            // Phenotype
            -25 => EntityType::Phenotype,
            // Gene (typically -1 or specific taxids for genes)
            -1 => EntityType::Gene,
            // Organism
            -30 => EntityType::Organism,
            // Unknown
            _ => EntityType::Unknown,
        }
    }

    /// Get the canonical string name for this entity type.
    ///
    /// # Examples
    ///
    /// ```
    /// use jensenlab_textmining_rs::taxonomy::EntityType;
    ///
    /// assert_eq!(EntityType::Protein.as_str(), "protein");
    /// assert_eq!(EntityType::Compartment.as_str(), "compartment");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Protein => "protein",
            EntityType::Disease => "disease",
            EntityType::Chemical => "chemical",
            EntityType::Tissue => "tissue",
            EntityType::Environment => "environment",
            EntityType::Compartment => "compartment",
            EntityType::BiologicalProcess => "biological_process",
            EntityType::Phenotype => "phenotype",
            EntityType::Gene => "gene",
            EntityType::Organism => "organism",
            EntityType::Unknown => "unknown",
        }
    }

    /// Get the numeric code for this entity type (if canonical).
    ///
    /// Returns the most common representative code for each type.
    /// Note: Some types like `Protein` have multiple codes (different taxids).
    #[must_use]
    pub fn canonical_code(&self) -> Option<i64> {
        match self {
            EntityType::Protein => Some(9606), // Human proteins as canonical
            EntityType::Disease => Some(-26),
            EntityType::Chemical => Some(-27),
            EntityType::Tissue => Some(-28),
            EntityType::Environment => Some(-29),
            EntityType::Compartment => Some(-22),
            EntityType::BiologicalProcess => Some(-23),
            EntityType::Phenotype => Some(-25),
            EntityType::Gene => Some(-1),
            EntityType::Organism => Some(-30),
            EntityType::Unknown => None,
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_from_code() {
        assert_eq!(EntityType::from_code(9606), EntityType::Protein);
        assert_eq!(EntityType::from_code(10090), EntityType::Protein); // Mouse
        assert_eq!(EntityType::from_code(-22), EntityType::Compartment);
        assert_eq!(EntityType::from_code(-26), EntityType::Disease);
        assert_eq!(EntityType::from_code(-27), EntityType::Chemical);
        assert_eq!(EntityType::from_code(-28), EntityType::Tissue);
        assert_eq!(EntityType::from_code(-29), EntityType::Environment);
        assert_eq!(EntityType::from_code(-23), EntityType::BiologicalProcess);
        assert_eq!(EntityType::from_code(-25), EntityType::Phenotype);
        assert_eq!(EntityType::from_code(-1), EntityType::Gene);
        assert_eq!(EntityType::from_code(-30), EntityType::Organism);
        assert_eq!(EntityType::from_code(99999), EntityType::Unknown);
    }

    #[test]
    fn test_entity_type_as_str() {
        assert_eq!(EntityType::Protein.as_str(), "protein");
        assert_eq!(EntityType::Compartment.as_str(), "compartment");
        assert_eq!(EntityType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_entity_type_display() {
        assert_eq!(format!("{}", EntityType::Protein), "protein");
        assert_eq!(format!("{}", EntityType::Disease), "disease");
    }
}
