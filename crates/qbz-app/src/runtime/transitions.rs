use super::manager::RuntimeManager;
use super::RuntimeState;

impl RuntimeManager {
    /// Update runtime state
    pub async fn set_state(&self, new_state: RuntimeState) {
        let mut status = self.state.write().await;
        status.state = new_state.clone();

        // Update derived fields based on state
        match &new_state {
            RuntimeState::Uninitialized => {
                status.client_initialized = false;
                status.legacy_auth = false;
                status.corebridge_auth = false;
                status.session_activated = false;
                status.user_id = None;
                status.degraded_reason = None;
            }
            RuntimeState::InitializedNoAuth => {
                status.client_initialized = true;
                status.legacy_auth = false;
                status.corebridge_auth = false;
                status.session_activated = false;
                status.user_id = None;
                status.degraded_reason = None;
            }
            RuntimeState::AuthenticatedNoUserSession { user_id } => {
                status.client_initialized = true;
                status.legacy_auth = true;
                status.session_activated = false;
                status.user_id = Some(*user_id);
                status.degraded_reason = None;
            }
            RuntimeState::Ready { user_id } => {
                status.client_initialized = true;
                status.legacy_auth = true;
                status.corebridge_auth = true;
                status.session_activated = true;
                status.user_id = Some(*user_id);
                status.degraded_reason = None;
            }
            RuntimeState::Degraded { reason } => {
                status.degraded_reason = Some(reason.clone());
            }
        }

        log::info!("[Runtime] State changed to: {:?}", new_state);
    }

    /// Mark client as initialized
    pub async fn set_client_initialized(&self, initialized: bool) {
        let mut status = self.state.write().await;
        status.client_initialized = initialized;
        if initialized && status.state == RuntimeState::Uninitialized {
            status.state = RuntimeState::InitializedNoAuth;
        }
    }

    /// Mark legacy auth status
    pub async fn set_legacy_auth(&self, auth: bool, user_id: Option<u64>) {
        let mut status = self.state.write().await;
        status.legacy_auth = auth;
        if auth {
            if let Some(uid) = user_id {
                status.user_id = Some(uid);
                if !status.session_activated {
                    status.state = RuntimeState::AuthenticatedNoUserSession { user_id: uid };
                }
            }
        } else {
            status.user_id = None;
            status.corebridge_auth = false;
            status.session_activated = false;
            status.state = RuntimeState::InitializedNoAuth;
        }
    }

    /// Mark CoreBridge auth status
    pub async fn set_corebridge_auth(&self, auth: bool) {
        let mut status = self.state.write().await;
        status.corebridge_auth = auth;
    }

    /// Mark session as activated
    pub async fn set_session_activated(&self, activated: bool, user_id: u64) {
        let mut status = self.state.write().await;
        status.session_activated = activated;
        if activated && status.legacy_auth && status.corebridge_auth {
            status.state = RuntimeState::Ready { user_id };
            status.user_id = Some(user_id);
        }
    }
}
