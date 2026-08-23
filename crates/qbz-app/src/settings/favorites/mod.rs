mod ops;
mod prefs;
mod schema;
mod state;
mod store;
#[cfg(test)]
mod tests;

pub use prefs::FavoritesPreferences;
pub use schema::{create_table, load_preferences, save_preferences};
pub use state::FavoritesPreferencesState;
pub use store::FavoritesPreferencesStore;
