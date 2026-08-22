# crates/qbz-credentials/src/lib.rs (1236 lines)

Secure credential storage: AES-256-GCM encrypted file (root-parameterized
for desktop + daemon), optional OS-keyring accelerator with a broken-state
latch + timeout, legacy XOR migration, and OAuth-token persistence.

## Proposed split

- `lib.rs` (~50 lines) — re-export surface (`save_qobuz_credentials`,
  `load_qobuz_credentials`, `has_saved_credentials`,
  `clear_qobuz_credentials`, `save_oauth_token*`, `load_oauth_token*`,
  `clear_oauth_token*`, `oauth_token_file_present_at`), `QobuzCredentials`
  struct, top-level consts.
- `paths.rs` (~90 lines) — `write_private_file`, `tighten_private_file_mode`
  (+ non-unix stub), the file-name consts, `get_fallback_path`,
  `get_legacy_fallback_path`, `config_qbz_root`,
  `installation_salt_path_at`, `machine_id_fallback_path_at`,
  `oauth_token_path_at`.
- `keys.rs` (~140 lines) — `load_or_create_installation_salt_at`,
  `load_or_create_machine_id_fallback_at`, `machine_id_stable_source`,
  `get_machine_id_at`, `get_portal_secret` (linux), `PortalKey` enum,
  `derive_key_at`.
- `crypto.rs` (~90 lines) — `EncryptedCredentials`, `encrypt_credentials*`,
  `decrypt_credentials*`, `legacy_deobfuscate`,
  `LEGACY_OBFUSCATION_KEY`.
- `fallback_file.rs` (~140 lines) — `load_legacy_credentials`,
  `save_to_fallback`, `load_from_fallback`, `clear_fallback`,
  `has_fallback_credentials`.
- `keyring.rs` (~150 lines) — `KEYRING_STATE`/consts,
  `keyring_is_broken`, `mark_keyring_broken`, `mark_keyring_working`,
  `run_with_keyring_timeout`, `keyring_get`, `keyring_set`,
  `keyring_delete`.
- `qobuz_credentials.rs` (~60 lines) — `save_qobuz_credentials`,
  `load_qobuz_credentials`, `has_saved_credentials`,
  `clear_qobuz_credentials` (the public Qobuz-specific API, thin
  orchestration over the above).
- `oauth_token.rs` (~180 lines) — all OAuth-token fns:
  `write_oauth_token_file`, `read_oauth_token_file`,
  `oauth_token_file_present_at`, `remove_oauth_token_file`,
  `save/load/clear_oauth_token_at` (daemon), `save/load/clear_oauth_token`
  (desktop), `load_oauth_token_from_file`.
- `tests.rs` (~200 lines) — existing test module (keep together since many
  tests exercise cross-module behavior: encryption roundtrip, keyring,
  file permissions).

## Tricky coupling — the big one

- `derive_key_at` (keys.rs) is called by both `crypto.rs`
  (`encrypt/decrypt_credentials_at`) and indirectly used everywhere — keep
  `PortalKey` enum in `keys.rs` and import it wherever needed.
- The desktop `save_oauth_token`/`load_oauth_token` (oauth_token.rs) layer
  the keyring accelerator (`keyring.rs`) on top of the file-first ops
  (`write_oauth_token_file`/`read_oauth_token_file`) — this cross-file call
  chain must stay intact (file first, keyring best-effort second).
- Root-parameterized fns (`*_at` suffix) exist specifically for
  desktop-vs-daemon profile separation (`PortalKey::Session` vs
  `PortalKey::Never`) — do not accidentally collapse these variants when
  splitting.

## Verify after split

`cargo build -p qbz-credentials`, `cargo test -p qbz-credentials` (roundtrip,
daemon-token-independence, presence-vs-decryptability, file-mode tests all
must stay green — several are gated on `has_writable_config_dir()` / not
CI, so run locally too if possible).
