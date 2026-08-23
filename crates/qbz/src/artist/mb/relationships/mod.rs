mod group;

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use slint::{ComponentHandle, ModelRc, VecModel};

use group::group_relations;

use crate::{AppWindow, MbRelationship, MbRelationshipsData, NetworkSidebarState};

/// Plain, `Send` mapped relationships ready to push into Slint. Members
/// here are the still-active ones (ended members already moved to
/// past_members on the qbz-core side, and Tauri's sidebar renders only
/// members — see groupedMembers in ArtistDetailView).
pub struct MbRelationshipsRowData {
    pub members: Vec<MbRelationshipRow>,
    pub groups: Vec<MbRelationshipRow>,
    pub collaborators: Vec<MbRelationshipRow>,
    pub has_data: bool,
}

pub struct MbRelationshipRow {
    pub mbid: String,
    pub name: String,
    /// Primary role for the musician-click callback. Defaults to "Band
    /// Member" / "Band" / "Collaborator" by section when MB has no
    /// attributes for the relation.
    pub role: String,
    /// Tooltip — roles joined with ", " plus the period in parens when
    /// present. Falls back to the period string or the name.
    pub tooltip: String,
}

/// Fetch MB relationships for `mbid` and map into the Slint-friendly
/// row shape. Groups members by mbid combining their roles, mirroring
/// Tauri's `groupMembersByMbid` plus the per-section role defaults.
pub async fn load_mb_relationships<A>(
    runtime: &Arc<AppRuntime<A>>,
    mbid: &str,
) -> Result<MbRelationshipsRowData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let relations = runtime
        .core()
        .musicbrainz_get_artist_relationships(mbid)
        .await
        .map_err(|e| e.to_string())?;
    Ok(map_relationships(relations))
}

fn map_relationships(
    rels: qbz_integrations::musicbrainz::ArtistRelationships,
) -> MbRelationshipsRowData {
    let members = group_relations(rels.members, "Band Member");
    let groups = group_relations(rels.groups, "Band");
    let collaborators = group_relations(rels.collaborators, "Collaborator");
    let has_data =
        !members.is_empty() || !groups.is_empty() || !collaborators.is_empty();
    MbRelationshipsRowData {
        members,
        groups,
        collaborators,
        has_data,
    }
}

/// Apply MB relationships to NetworkSidebarState. Runs on the Slint
/// event loop.
pub fn apply_mb_relationships(window: &AppWindow, data: MbRelationshipsRowData) {
    let to_slint = |rows: Vec<MbRelationshipRow>| -> ModelRc<MbRelationship> {
        ModelRc::new(VecModel::from(
            rows.into_iter()
                .map(|r| MbRelationship {
                    mbid: r.mbid.into(),
                    name: r.name.into(),
                    role: r.role.into(),
                    tooltip: r.tooltip.into(),
                })
                .collect::<Vec<_>>(),
        ))
    };
    let state = window.global::<NetworkSidebarState>();
    state.set_relationships(MbRelationshipsData {
        members: to_slint(data.members),
        groups: to_slint(data.groups),
        collaborators: to_slint(data.collaborators),
        has_data: data.has_data,
    });
    state.set_relationships_loading(false);
}
