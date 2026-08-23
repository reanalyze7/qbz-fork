use super::*;

#[tokio::test]
async fn test_mixtape_context_set_clear() {
    let mgr = RuntimeManager::new();

    // Initially None
    assert_eq!(mgr.get_queue_source_collection().await, None);

    // Set Some
    mgr.set_queue_source_collection(Some("col-abc".to_string())).await;
    assert_eq!(
        mgr.get_queue_source_collection().await,
        Some("col-abc".to_string())
    );

    // Clear to None
    mgr.set_queue_source_collection(None).await;
    assert_eq!(mgr.get_queue_source_collection().await, None);
}

#[tokio::test]
async fn default_state_is_uninitialized() {
    let mgr = RuntimeManager::new();
    let status = mgr.get_status().await;

    assert_eq!(status.state, RuntimeState::Uninitialized);
    assert!(!status.client_initialized);
    assert!(!status.legacy_auth);
    assert!(!status.corebridge_auth);
    assert!(!status.session_activated);
    assert_eq!(status.user_id, None);
}

#[tokio::test]
async fn client_initialized_transitions_to_initialized_no_auth() {
    let mgr = RuntimeManager::new();

    mgr.set_client_initialized(true).await;
    let status = mgr.get_status().await;

    assert_eq!(status.state, RuntimeState::InitializedNoAuth);
    assert!(status.client_initialized);
    assert!(!status.legacy_auth);
    assert_eq!(status.user_id, None);
}

#[tokio::test]
async fn legacy_auth_transitions_to_authenticated_no_user_session() {
    let mgr = RuntimeManager::new();

    mgr.set_client_initialized(true).await;
    mgr.set_legacy_auth(true, Some(42)).await;
    let status = mgr.get_status().await;

    assert_eq!(
        status.state,
        RuntimeState::AuthenticatedNoUserSession { user_id: 42 }
    );
    assert!(status.client_initialized);
    assert!(status.legacy_auth);
    assert!(!status.session_activated);
    assert_eq!(status.user_id, Some(42));
}

#[tokio::test]
async fn corebridge_auth_and_session_activation_reaches_ready() {
    let mgr = RuntimeManager::new();

    mgr.set_client_initialized(true).await;
    mgr.set_legacy_auth(true, Some(42)).await;
    mgr.set_corebridge_auth(true).await;
    mgr.set_session_activated(true, 42).await;
    let status = mgr.get_status().await;

    assert_eq!(status.state, RuntimeState::Ready { user_id: 42 });
    assert!(status.client_initialized);
    assert!(status.legacy_auth);
    assert!(status.corebridge_auth);
    assert!(status.session_activated);
    assert_eq!(status.user_id, Some(42));
}

#[tokio::test]
async fn requires_user_session_accepts_offline_session_without_legacy_auth() {
    let mgr = RuntimeManager::new();

    mgr.set_session_activated(true, 0).await;

    assert!(mgr
        .check_requirements(CommandRequirement::RequiresUserSession)
        .await
        .is_ok());
    assert!(matches!(
        mgr.check_requirements(CommandRequirement::RequiresAuth).await,
        Err(RuntimeError::RuntimeNotInitialized)
    ));
}
