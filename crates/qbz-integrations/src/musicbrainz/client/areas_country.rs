use super::MusicBrainzClient;
use crate::error::IntegrationResult;

impl MusicBrainzClient {
    /// Resolve the country that an area belongs to by walking up the hierarchy.
    ///
    /// Frankfurt am Main → Hessen → Germany → returns ("Germany", Some("de"))
    /// Monterrey → Nuevo León → Mexico → returns ("Mexico", Some("mx"))
    ///
    /// Returns (country_name, country_code_lowercase) or None.
    pub async fn resolve_area_country(
        &self,
        area_id: &str,
    ) -> IntegrationResult<Option<(String, Option<String>)>> {
        let mut current_id = area_id.to_string();
        let max_hops = 5;

        for _hop in 0..max_hops {
            let detail = self.get_area_with_relations(&current_id).await?;

            // If current area IS a country, return it
            if detail
                .area_type
                .as_deref()
                .map(|t| t.eq_ignore_ascii_case("country"))
                .unwrap_or(false)
            {
                let code = detail
                    .iso_codes
                    .as_ref()
                    .and_then(|c| c.first())
                    .map(|c| c.to_lowercase());
                return Ok(Some((detail.name, code)));
            }

            // Find "part of" parent
            let parent = detail
                .relations
                .as_ref()
                .and_then(|rels| {
                    rels.iter()
                        .find(|rel| {
                            rel.relation_type == "part of"
                                && rel.direction.as_deref() == Some("backward")
                        })
                        .and_then(|rel| rel.area.as_ref())
                });

            match parent {
                Some(p) => {
                    if p.area_type
                        .as_deref()
                        .map(|t| t.eq_ignore_ascii_case("country"))
                        .unwrap_or(false)
                    {
                        let code = p
                            .iso_codes
                            .as_ref()
                            .and_then(|c| c.first())
                            .map(|c| c.to_lowercase());
                        return Ok(Some((p.name.clone(), code)));
                    }
                    current_id = p.id.clone();
                }
                None => return Ok(None),
            }
        }

        Ok(None)
    }
}
