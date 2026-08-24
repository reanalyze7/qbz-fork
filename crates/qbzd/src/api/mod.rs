// crates/qbzd/src/api/mod.rs — the HTTP control plane (02-cli-and-api.md §3).
//
// tiny_http 0.12, single listener, thread-per-connection collapsed to ONE
// serving thread (requests are handled inline, serialized — a control plane
// sees a handful of clients, never a load). Two mechanisms qualify the
// otherwise-open surface (§3.1.2):
//   - Origin shield (ALWAYS on): any request carrying an `Origin` header is
//     refused `403 origin_forbidden` before routing — CLI/curl/scripts send no
//     `Origin`, browsers do (CSRF / DNS-rebinding guard at zero UX cost).
//   - Opt-in `[server] token`: when set, `Authorization: Bearer <token>` is
//     required on every route EXCEPT `GET /api/ping`; a mismatch is
//     `401 invalid_token`. Unset = no auth machinery exists at runtime.
//
// The two-call split — `bind` at boot step 5 (stateless, so the foreign-occupant
// diagnosis runs BEFORE the stores/runtime exist), `serve` at boot step 11 — is
// what keeps the 01-architecture.md §8.1 boot order honest.
pub mod browse;
pub mod discover;
pub mod fav;
pub mod play;
pub mod playback;
pub mod playlist;
pub mod queue;
pub mod reco;
pub mod search;
pub mod artwork;
pub mod settings;
pub mod sse;
pub mod status;

mod gate;
mod lifecycle;
mod response;
mod router;
mod router_lists;
mod routes_table;
mod state;
#[cfg(test)]
mod tests;

pub use lifecycle::{bind, probe_is_qbzd, serve};
pub use state::{ApiHandle, ApiState, BindError, BoundServer, DeviceCache};

pub(crate) use response::{canon_volume, err_json, json};
