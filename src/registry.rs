//! Data-driven registry over [`crate::scores::ALL`].
//!
//! All listing/introspection (`list_scores`, `score_inputs`, `suggest_scores`) is derived
//! from the descriptors here — no metadata is duplicated.

use crate::scores::ALL;
use crate::ScoreDescriptor;

/// All registered scores.
pub fn all() -> &'static [ScoreDescriptor] {
    ALL
}

/// Look up a score by id.
pub fn find(id: &str) -> Option<&'static ScoreDescriptor> {
    ALL.iter().find(|d| d.id == id)
}

/// All distinct domains, in first-seen order.
pub fn domains() -> Vec<&'static str> {
    let mut seen = Vec::new();
    for d in ALL {
        if !seen.contains(&d.domain) {
            seen.push(d.domain);
        }
    }
    seen
}

/// Filter by optional domain and a free-text query matched against id/name/keywords.
pub fn filter(domain: Option<&str>, query: Option<&str>) -> Vec<&'static ScoreDescriptor> {
    let q = query.map(|s| s.to_ascii_lowercase());
    ALL.iter()
        .filter(|d| domain.map(|dm| d.domain == dm).unwrap_or(true))
        .filter(|d| match &q {
            None => true,
            Some(q) => {
                d.id.to_ascii_lowercase().contains(q)
                    || d.name.to_ascii_lowercase().contains(q)
                    || d.keywords
                        .iter()
                        .any(|k| k.to_ascii_lowercase().contains(q))
            }
        })
        .collect()
}
