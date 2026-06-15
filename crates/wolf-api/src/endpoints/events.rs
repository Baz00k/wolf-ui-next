use futures_util::StreamExt;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::{ApiError, WolfApi, types};

#[derive(Clone, Debug, PartialEq)]
pub enum WolfEvent {
    LobbyCreated(Box<types::RflReflectorWolfCoreEventsLobbyReflType>),
    LobbyJoined(types::WolfCoreEventsJoinLobbyEvent),
    LobbyLeft(types::WolfCoreEventsLeaveLobbyEvent),
    LobbyStopped(String),
    Other(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct CreateLobbyEvent {
    id: String,
    profile_id: String,
    name: String,
    icon_png_path: Option<String>,
    pin: Option<Vec<i64>>,
    multi_user: bool,
    stop_when_everyone_leaves: bool,
    runner: types::RflReflectorWolfCoreEventsLobbyReflTypeRunner,
}

#[derive(Clone, Copy, Debug)]
pub struct Events<'api> {
    api: &'api WolfApi,
}

impl<'api> Events<'api> {
    pub(crate) fn new(api: &'api WolfApi) -> Self {
        Self { api }
    }

    pub async fn listen<F>(&self, mut on_event: F) -> Result<(), ApiError>
    where
        F: FnMut(WolfEvent),
    {
        let path = "/api/v1/events";
        let context = self.api.request_context("GET", path);
        let response = self.api.get_stream_response(path).await?;
        let mut stream = response.bytes_stream();
        let mut parser = EventStreamParser::default();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            buffer.push_str(&String::from_utf8_lossy(&chunk.map_err(|error| {
                let error = ApiError::from_reqwest(error);
                self.api
                    .log_api_error(context, &error, "Wolf API event stream failed");
                error
            })?));

            while let Some(index) = buffer.find('\n') {
                let line = buffer[..index].trim_end_matches('\r').to_string();
                buffer.replace_range(..=index, "");

                if let Some(event) = parser.accept_line(&line) {
                    match event {
                        Ok(event) => on_event(event),
                        Err(error) => {
                            self.api
                                .log_api_error(context, &error, "Wolf API event decode failed")
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct EventStreamParser {
    event_type: String,
    data: String,
}

impl EventStreamParser {
    fn accept_line(&mut self, line: &str) -> Option<Result<WolfEvent, ApiError>> {
        if line.is_empty() {
            let event = (!self.event_type.is_empty())
                .then(|| parse_event(self.event_type.clone(), self.data.clone()));
            self.event_type.clear();
            self.data.clear();
            return event;
        }

        if let Some(value) = line.strip_prefix("event: ") {
            self.event_type = value.to_string();
        } else if let Some(value) = line.strip_prefix("data: ") {
            self.data.push_str(value);
        }

        None
    }
}

fn parse_event(event: String, data: String) -> Result<WolfEvent, ApiError> {
    match event.as_str() {
        "wolf::core::events::CreateLobbyEvent" => parse_json::<CreateLobbyEvent>(&data)
            .map(CreateLobbyEvent::into_lobby)
            .map(Box::new)
            .map(WolfEvent::LobbyCreated),
        "wolf::core::events::JoinLobbyEvent" => parse_json(&data).map(WolfEvent::LobbyJoined),
        "wolf::core::events::LeaveLobbyEvent" => parse_json(&data).map(WolfEvent::LobbyLeft),
        "wolf::core::events::StopLobbyEvent" => {
            parse_json::<types::WolfCoreEventsStopLobbyEvent>(&data)
                .map(|event| WolfEvent::LobbyStopped(event.lobby_id))
        }
        _ => Ok(WolfEvent::Other(event)),
    }
}

impl CreateLobbyEvent {
    fn into_lobby(self) -> types::RflReflectorWolfCoreEventsLobbyReflType {
        types::RflReflectorWolfCoreEventsLobbyReflType {
            connected_sessions: Vec::new(),
            icon_png_path: self.icon_png_path,
            id: self.id,
            multi_user: self.multi_user,
            name: self.name,
            pin_required: self.pin.is_some(),
            runner: self.runner,
            started_by_profile_id: self.profile_id,
            stop_when_everyone_leaves: self.stop_when_everyone_leaves,
        }
    }
}

fn parse_json<T>(data: &str) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    serde_json::from_str(data).map_err(ApiError::from_serde_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_emits_and_resets_unknown_events_on_blank_lines() {
        let mut parser = EventStreamParser::default();

        assert!(parser.accept_line("event: wolf::unknown").is_none());
        assert!(parser.accept_line("data: ignored").is_none());

        let event = parser
            .accept_line("")
            .expect("blank line should complete the event")
            .expect("unknown event should parse");

        assert_eq!(event, WolfEvent::Other("wolf::unknown".to_string()));

        parser.accept_line("event: second");
        let second = parser
            .accept_line("")
            .expect("blank line should complete second event")
            .expect("second event should parse");

        assert_eq!(second, WolfEvent::Other("second".to_string()));
    }

    #[test]
    fn parser_returns_decode_errors_for_malformed_known_events() {
        let mut parser = EventStreamParser::default();

        parser.accept_line("event: wolf::core::events::StopLobbyEvent");
        parser.accept_line("data: not-json");

        let error = parser
            .accept_line("")
            .expect("blank line should complete the event")
            .expect_err("malformed known event should fail");

        assert!(matches!(error, ApiError::EventDecode(_)));
    }

    #[test]
    fn parser_maps_create_lobby_events_into_lobby_state() {
        let event = parse_event(
            "wolf::core::events::CreateLobbyEvent".to_string(),
            r#"{"id":"lobby","profile_id":"profile","name":"Game","multi_user":true,"pin":null,"stop_when_everyone_leaves":true,"runner":{"type":"process","run_cmd":"game"},"video_settings":{"width":1920,"height":1080,"refresh_rate":60,"wayland_render_node":"software","runner_render_node":"render","video_producer_buffer_caps":"video/x-raw"},"audio_settings":{"channel_count":2},"client_settings":{},"runner_state_folder":"state"}"#.to_string(),
        )
        .expect("create lobby event should map into lobby state");

        match event {
            WolfEvent::LobbyCreated(lobby) => {
                assert_eq!(lobby.id, "lobby");
                assert_eq!(lobby.started_by_profile_id, "profile");
                assert!(lobby.connected_sessions.is_empty());
            }
            _ => panic!("expected lobby created event"),
        }
    }
}
