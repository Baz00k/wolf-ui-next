use futures_util::StreamExt;
use serde::de::DeserializeOwned;

use crate::{ApiError, WolfApi, types};

#[derive(Clone, Debug, PartialEq)]
pub enum WolfEvent {
    LobbyCreated(Box<types::RflReflectorWolfCoreEventsLobbyReflType>),
    LobbyStopped(String),
    Other(String),
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
        let response = self.api.get_response(path).await?;
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
        "wolf::core::events::CreateLobbyEvent" => {
            parse_json(&data).map(|event| WolfEvent::LobbyCreated(Box::new(event)))
        }
        "wolf::core::events::StopLobbyEvent" => {
            parse_json::<types::WolfCoreEventsStopLobbyEvent>(&data)
                .map(|event| WolfEvent::LobbyStopped(event.lobby_id))
        }
        _ => Ok(WolfEvent::Other(event)),
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
    fn parser_ignores_data_without_event_type() {
        let mut parser = EventStreamParser::default();

        assert!(parser.accept_line("data: ignored").is_none());
        assert!(parser.accept_line("").is_none());
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
}
