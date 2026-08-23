use super::error::RuntimeError;
use super::RuntimeStatus;

/// Command prerequisites - what each command requires
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandRequirement {
    /// No requirements (public endpoints)
    None,
    /// Requires client to be initialized (bundle tokens)
    RequiresClientInit,
    /// Requires authentication (logged in)
    RequiresAuth,
    /// Requires per-user session to be activated
    RequiresUserSession,
    /// Requires CoreBridge/V2 auth (for V2 commands)
    RequiresCoreBridgeAuth,
}

/// Validate `req` against `status`. Split out of `RuntimeManager` so the
/// state-machine transitions (in `manager.rs`) and the read-only requirement
/// gating logic can live in separate files without a circular dependency.
pub(super) fn check(status: &RuntimeStatus, req: CommandRequirement) -> Result<(), RuntimeError> {
    match req {
        CommandRequirement::None => Ok(()),
        CommandRequirement::RequiresClientInit => {
            if !status.client_initialized {
                Err(RuntimeError::RuntimeNotInitialized)
            } else {
                Ok(())
            }
        }
        CommandRequirement::RequiresAuth => {
            if !status.client_initialized {
                Err(RuntimeError::RuntimeNotInitialized)
            } else if !status.legacy_auth {
                Err(RuntimeError::AuthRequired)
            } else {
                Ok(())
            }
        }
        CommandRequirement::RequiresUserSession => {
            // RequiresUserSession only checks session_activated, not legacy_auth.
            // This allows both:
            // - Online sessions: session_activated=true via activate_session()
            // - Offline sessions: session_activated=true via activate_offline_session()
            //
            // Commands needing Qobuz API should use RequiresAuth or RequiresCoreBridgeAuth.
            if !status.session_activated {
                Err(RuntimeError::UserSessionNotActivated)
            } else {
                Ok(())
            }
        }
        CommandRequirement::RequiresCoreBridgeAuth => {
            if !status.client_initialized {
                Err(RuntimeError::RuntimeNotInitialized)
            } else if !status.legacy_auth {
                Err(RuntimeError::AuthRequired)
            } else if !status.session_activated {
                Err(RuntimeError::UserSessionNotActivated)
            } else if !status.corebridge_auth {
                Err(RuntimeError::CoreBridgeAuthMissing)
            } else {
                Ok(())
            }
        }
    }
}
