use dioxus::prelude::*;
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::ProfileListResponse;

use crate::api::ApiContext;

#[derive(Clone, PartialEq)]
pub(crate) struct ProfilesState {
    pub profiles: ProfileListResponse,
    pub lobbies: Vec<Lobby>,
}

pub(crate) async fn load_profiles_state() -> Result<ProfilesState, String> {
    let api = ApiContext::consume();
    let profiles = api.profiles().list().await.map_err(|_| {
        "Profiles could not be loaded. Check that Wolf is running, then try again.".to_string()
    })?;
    let lobbies = api.lobbies().list().await.map_err(|_| {
        "Profiles could not be loaded. Check that Wolf is running, then try again.".to_string()
    })?;

    Ok(ProfilesState {
        profiles,
        lobbies: multi_user_lobbies(lobbies.lobbies),
    })
}

pub(crate) async fn listen_for_lobby_events(lobbies: Signal<Vec<Lobby>>) {
    let _ = ApiContext::consume()
        .events()
        .listen(|event| apply_lobby_event(event, lobbies))
        .await;
}

fn multi_user_lobbies(lobbies: Vec<Lobby>) -> Vec<Lobby> {
    lobbies
        .into_iter()
        .filter(|lobby| lobby.multi_user)
        .collect()
}

fn apply_lobby_event(event: wolf_api::events::WolfEvent, mut lobbies: Signal<Vec<Lobby>>) {
    match event {
        wolf_api::events::WolfEvent::LobbyCreated(lobby) if lobby.multi_user => {
            lobbies.with_mut(|items| {
                items.retain(|item| item.id != lobby.id);
                items.push(*lobby);
            });
        }
        wolf_api::events::WolfEvent::LobbyStopped(lobby_id) => {
            lobbies.with_mut(|items| items.retain(|item| item.id != lobby_id));
        }
        wolf_api::events::WolfEvent::LobbyJoined(event) => {
            let session_id = event.moonlight_session_id;
            lobbies.with_mut(|items| {
                if let Some(lobby) = items.iter_mut().find(|lobby| lobby.id == event.lobby_id)
                    && !lobby.connected_sessions.contains(&session_id)
                {
                    lobby.connected_sessions.push(session_id.clone());
                }
            });
        }
        wolf_api::events::WolfEvent::LobbyLeft(event) => {
            let session_id = event.moonlight_session_id;
            lobbies.with_mut(|items| {
                if let Some(lobby) = items.iter_mut().find(|lobby| lobby.id == event.lobby_id) {
                    lobby
                        .connected_sessions
                        .retain(|session| session != &session_id);
                }
            });
        }
        _ => {}
    }
}
