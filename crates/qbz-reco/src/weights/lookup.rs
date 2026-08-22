//! Lookup helpers mapping relationship/source strings to weight values.

use super::RelationshipWeights;

impl RelationshipWeights {
    /// Get weight for a MusicBrainz relationship type
    pub fn weight_for_mb_relation(&self, relation_type: &str) -> f32 {
        match relation_type.to_lowercase().as_str() {
            // Band relationships
            "member of band" | "member_of_band" => self.member_of_band,
            "has member" | "has_member" => self.has_member,
            "founder" | "founded" => self.founder,

            // Collaboration
            "collaboration" | "collaborated" | "collaborator" => self.collaboration,

            // Performance credits
            "performer"
            | "vocal"
            | "instrument"
            | "performing orchestra"
            | "orchestra"
            | "chorus master" => self.performer,

            // Composition
            "composer" | "writer" | "lyricist" | "librettist" | "arranger" => self.composer,

            // Production
            "producer" | "executive producer" | "co-producer" => self.producer,
            "conductor" => self.conductor,
            "engineer" | "mix" | "mixer" | "mastering" | "recording" => self.engineer,

            // Unknown - use small default weight
            _ => 0.2,
        }
    }

    /// Get weight for a source type string
    pub fn weight_for_source(&self, source: &str) -> f32 {
        match source {
            "qobuz_similar" => self.qobuz_similar,
            "shared_tag" => self.shared_tag,
            "user_affinity" => self.user_affinity,
            s if s.starts_with("mb:") => {
                let rel_type = &s[3..];
                self.weight_for_mb_relation(rel_type)
            }
            _ => 0.2,
        }
    }
}
