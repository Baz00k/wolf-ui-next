pub(crate) const WOLF_SESSION_ID_ENV: &str = "WOLF_SESSION_ID";

pub(crate) fn session_id_from_env(
    get_env: impl FnOnce(&str) -> Option<String>,
) -> Result<String, String> {
    get_env(WOLF_SESSION_ID_ENV)
        .filter(|session_id| !session_id.trim().is_empty())
        .ok_or_else(|| format!("{WOLF_SESSION_ID_ENV} is not set."))
}

pub(crate) fn current_session_id() -> Result<String, String> {
    session_id_from_env(|key| std::env::var(key).ok())
}

pub(crate) async fn stop_current_session() -> Result<(), std::io::Error> {
    let api = crate::api::ApiContext::consume();
    let session_id = current_session_id().map_err(std::io::Error::other)?;
    let response = api
        .sessions()
        .stop(&session_id)
        .await
        .map_err(std::io::Error::other)?;

    if !response.success {
        return Err(std::io::Error::other("Wolf did not stop the session."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{WOLF_SESSION_ID_ENV, session_id_from_env};

    #[test]
    fn session_id_from_env_reads_wolf_session_id() {
        let session_id = session_id_from_env(|key| {
            assert_eq!(key, WOLF_SESSION_ID_ENV);
            Some("moonlight-session-7".to_string())
        })
        .expect("session id is present");

        assert_eq!(session_id, "moonlight-session-7");
    }

    #[test]
    fn session_id_from_env_rejects_empty_values() {
        let error = session_id_from_env(|_| Some("   ".to_string()))
            .expect_err("blank session id is rejected");

        assert_eq!(error, "WOLF_SESSION_ID is not set.");
    }

    #[test]
    fn session_id_from_env_rejects_missing_values() {
        let error = session_id_from_env(|_| None).expect_err("missing session id is rejected");

        assert_eq!(error, "WOLF_SESSION_ID is not set.");
    }
}
