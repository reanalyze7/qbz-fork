/// Everything that can go wrong on the way to a persisted session. Every variant
/// renders with a `→` fix line (02 §1.4). All three map to exit 1 in `main`
/// (login never reports 3/4 — it does its own OAuth and local persist, so it
/// works daemon-up or daemon-down, §2.2).
#[derive(Debug)]
pub enum LoginError {
    /// No nonce-valid redirect arrived within [`LOGIN_DEADLINE`]. Carries the
    /// ephemeral port so the timeout copy can forward exactly it.
    Timeout(u16),
    /// Qobuz explicitly rejected the credentials (401 / ineligible account).
    Rejected(String),
    /// Any other local or network-class failure.
    Failed(String),
}

impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginError::Timeout(port) => write!(f, "{}", crate::cli::copy::login_timeout(*port)),
            LoginError::Rejected(msg) => write!(
                f,
                "error: Qobuz rejected the credentials ({msg})\n  \
                 → check the token or sign in again:  qbzd login"
            ),
            LoginError::Failed(msg) => {
                write!(f, "error: {msg}")?;
                if !msg.contains('→') {
                    write!(
                        f,
                        "\n  → check your connection and retry:  qbzd login\n  \
                         → or inject a token directly:      qbzd login --token <user_auth_token>"
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for LoginError {}

pub(super) fn map_api_err(e: qbz_qobuz::ApiError) -> LoginError {
    match e {
        qbz_qobuz::ApiError::AuthenticationError(_) | qbz_qobuz::ApiError::IneligibleUser => {
            LoginError::Rejected(e.to_string())
        }
        other => LoginError::Failed(other.to_string()),
    }
}

pub(super) fn map_core_err(e: qbz_core::CoreError) -> LoginError {
    if matches!(
        e,
        qbz_core::CoreError::Api(
            qbz_qobuz::ApiError::AuthenticationError(_) | qbz_qobuz::ApiError::IneligibleUser
        )
    ) {
        LoginError::Rejected(e.to_string())
    } else {
        LoginError::Failed(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_timeout_error_renders_the_verbatim_copy_with_the_port() {
        let rendered = LoginError::Timeout(39114).to_string();
        assert!(rendered.contains("no OAuth redirect received within 300 s"), "{rendered}");
        assert!(rendered.contains("ssh -L 39114:localhost:39114"), "{rendered}");
        assert!(rendered.contains("qbzd login --paste"), "{rendered}");
        assert!(rendered.contains("qbzd login --token <user_auth_token>"), "{rendered}");
    }
}
