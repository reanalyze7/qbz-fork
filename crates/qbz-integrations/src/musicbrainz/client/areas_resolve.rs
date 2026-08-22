use super::MusicBrainzClient;
use crate::error::IntegrationResult;

impl MusicBrainzClient {
    /// Resolve a city area to its parent subdivision (state/region)
    pub async fn resolve_parent_subdivision(
        &self,
        area_id: &str,
    ) -> IntegrationResult<Option<(String, String)>> {
        let mut current_id = area_id.to_string();
        let mut path: Vec<String> = Vec::new();
        let max_hops = 5;

        for _hop in 0..max_hops {
            let detail = self.get_area_with_relations(&current_id).await?;
            path.push(format!("{}[{:?}]", detail.name, detail.area_type));

            let parents: Vec<_> = detail
                .relations
                .as_ref()
                .map(|rels| {
                    rels.iter()
                        .filter(|rel| {
                            rel.relation_type == "part of"
                                && rel.direction.as_deref() == Some("backward")
                        })
                        .filter_map(|rel| rel.area.as_ref())
                        .collect()
                })
                .unwrap_or_default();

            if parents.is_empty() {
                return Ok(None);
            }

            let has_country_parent = parents.iter().any(|p| {
                p.area_type
                    .as_deref()
                    .map(|t| t.eq_ignore_ascii_case("country"))
                    .unwrap_or(false)
            });

            if has_country_parent {
                let own_type = detail.area_type.as_deref().unwrap_or("");
                if own_type.eq_ignore_ascii_case("subdivision") {
                    if current_id == area_id {
                        return Ok(None);
                    }
                    return Ok(Some((detail.name.clone(), detail.id.clone())));
                }
                if current_id == area_id {
                    return Ok(None);
                }
                return Ok(Some((detail.name.clone(), detail.id.clone())));
            }

            let next = parents
                .iter()
                .find(|p| {
                    p.area_type
                        .as_deref()
                        .map(|t| t.eq_ignore_ascii_case("subdivision"))
                        .unwrap_or(false)
                })
                .or_else(|| {
                    parents.iter().find(|p| {
                        let t = p.area_type.as_deref().unwrap_or("");
                        !t.eq_ignore_ascii_case("city") && !t.eq_ignore_ascii_case("country")
                    })
                })
                .or_else(|| parents.first());

            match next {
                Some(parent) => {
                    current_id = parent.id.clone();
                }
                None => {
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }
}
