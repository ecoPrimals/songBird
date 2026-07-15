// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! External service bond definitions (weak bonds to public APIs).
//!
//! These define the well-known external services that compositions may query
//! via the drawbridge external proxy. Each bond is a `(name, base_url)` pair
//! suitable for `SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST`.
//!
//! ## Bond Types
//!
//! - **GIS**: Geographic information services (footPrint composition)
//! - **Science**: Biological/chemical databases (petalTongue, loamSpine, ABG)

/// GIS weak bonds (footPrint composition).
pub const GIS_BONDS: &[(&str, &str)] = &[
    ("osm", "https://tile.openstreetmap.org"),
    ("overpass", "https://overpass-api.de"),
    ("fema", "https://hazards.fema.gov"),
    ("arcgis1", "https://services1.arcgis.com"),
    ("arcgis2", "https://services2.arcgis.com"),
    ("nominatim", "https://nominatim.openstreetmap.org"),
    ("usgs", "https://epqs.nationalmap.gov"),
    ("nrcs", "https://sdmdataaccess.sc.egov.usda.gov"),
    ("michigan", "https://gisagocss.state.mi.us"),
    ("mcgi", "https://gisp.mcgi.state.mi.us"),
    ("eastlansing", "https://gis2.cityofeastlansing.com"),
];

/// Science API weak bonds (petalTongue, loamSpine, ABG compositions).
pub const SCIENCE_BONDS: &[(&str, &str)] = &[
    ("ncbi", "https://eutils.ncbi.nlm.nih.gov"),
    ("pubchem", "https://pubchem.ncbi.nlm.nih.gov"),
    ("blast", "https://blast.ncbi.nlm.nih.gov"),
    ("uniprot", "https://rest.uniprot.org"),
    ("pdb", "https://data.rcsb.org"),
    ("alphafold", "https://alphafold.ebi.ac.uk"),
];

/// All known weak bonds (GIS + Science).
pub const ALL_BONDS: &[(&str, &str)] = &[
    // GIS
    ("osm", "https://tile.openstreetmap.org"),
    ("overpass", "https://overpass-api.de"),
    ("fema", "https://hazards.fema.gov"),
    ("arcgis1", "https://services1.arcgis.com"),
    ("arcgis2", "https://services2.arcgis.com"),
    ("nominatim", "https://nominatim.openstreetmap.org"),
    ("usgs", "https://epqs.nationalmap.gov"),
    ("nrcs", "https://sdmdataaccess.sc.egov.usda.gov"),
    ("michigan", "https://gisagocss.state.mi.us"),
    ("mcgi", "https://gisp.mcgi.state.mi.us"),
    ("eastlansing", "https://gis2.cityofeastlansing.com"),
    // Science
    ("ncbi", "https://eutils.ncbi.nlm.nih.gov"),
    ("pubchem", "https://pubchem.ncbi.nlm.nih.gov"),
    ("blast", "https://blast.ncbi.nlm.nih.gov"),
    ("uniprot", "https://rest.uniprot.org"),
    ("pdb", "https://data.rcsb.org"),
    ("alphafold", "https://alphafold.ebi.ac.uk"),
];

/// Format all bonds in a given slice as a `SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST`
/// environment variable value.
#[must_use]
pub fn format_allowlist(bonds: &[(&str, &str)]) -> String {
    bonds
        .iter()
        .map(|(name, url)| format!("{name}={url}"))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_science_allowlist() {
        let value = format_allowlist(SCIENCE_BONDS);
        assert!(value.contains("ncbi=https://eutils.ncbi.nlm.nih.gov"));
        assert!(value.contains("pubchem=https://pubchem.ncbi.nlm.nih.gov"));
        assert!(value.contains("uniprot=https://rest.uniprot.org"));
        assert_eq!(value.matches(',').count(), SCIENCE_BONDS.len() - 1);
    }

    #[test]
    fn format_all_bonds_allowlist() {
        let value = format_allowlist(ALL_BONDS);
        assert_eq!(value.matches(',').count(), ALL_BONDS.len() - 1);
        assert_eq!(ALL_BONDS.len(), GIS_BONDS.len() + SCIENCE_BONDS.len());
    }

    #[test]
    fn all_bonds_have_https() {
        for (name, url) in ALL_BONDS {
            assert!(
                url.starts_with("https://"),
                "Bond '{name}' must use HTTPS: {url}"
            );
        }
    }

    #[test]
    fn no_duplicate_bond_names() {
        let mut seen = std::collections::HashSet::new();
        for (name, _) in ALL_BONDS {
            assert!(seen.insert(name), "Duplicate bond name: {name}");
        }
    }
}
