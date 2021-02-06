// imports
use crate::frontend::helper::log_error;
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use derive_more::Display;
use serde::Serialize;

pub type WebResult<R> = Result<R, WebsocketError>;

#[derive(Debug, Display)]
pub enum WebsocketError {
    #[display(fmt = "Internal Error")]
    InternalError(String),
    #[display(fmt = "Failed to load message from JSON")]
    MessageFormatError(),
    #[display(fmt = "Failed to load message due to invalid content")]
    ValidationError(String),
    #[display(fmt = "Unimplemented feature")]
    UnimplementedError(),
    #[display(fmt = "You don't have the required permissions to perform this action")]
    AuthorizationError(),
}

#[derive(Serialize)]
pub struct ErrorMessage<'a> {
    code: u16,
    message: &'a str,
}

impl<'a> ErrorMessage<'a> {
    pub fn text<'x>(error: WebsocketError) -> String {
        let (code, message) = match error {
            WebsocketError::InternalError(e) => {
                (0, format!("Internal Error: {}", e).as_str())
            }
            WebsocketError::MessageFormatError { .. } => {
                (1, "MessageError: Seems like your message couldn't be loaded from JSON")
            }
            WebsocketError::AuthorizationError { .. } => {
                (3, "MessageError: Seems like you weren't allowed to perform this action")
            }
            WebsocketError::UnimplementedError { .. } => {
                (u16::MAX, "Unimplemented: The action you tried to use is either implemented/ supported at them moment")
            }
        };

        return serde_json::to_string(&ErrorMessage { code, message })
            .expect("The creation of websocket error messages failed");
    }
}

impl ResponseError for WebsocketError {
    fn error_response(&self) -> HttpResponse {
        match self {
            WebsocketError::InternalError(e) => {
                log_error("Internal Panic", e.clone());
                HttpResponse::Ok().json(ErrorMessage {
                    message: e.as_str(),
                    code: 0,
                })
            }
            _ => HttpResponse::Ok().json(ErrorMessage {
                message: ErrorMessage::text(*self).as_str(),
                code: self.status_code().as_u16(),
            }),
        }
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            WebsocketError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            WebsocketError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<serde_json::Error> for WebsocketError {
    fn from(_: serde_json::Error) -> Self {
        return WebsocketError::MessageFormatError {};
    }
}

// Errors with 'static' outcomes that don't feature failure specific fields may be cached here
lazy_static! {
    pub static ref MESSAGE_FORMAT_ERROR: String =
        ErrorMessage::text(WebsocketError::MessageFormatError());
    pub static ref UNIMPLEMENTED_ERROR: String =
        ErrorMessage::text(WebsocketError::UnimplementedError());
    pub static ref UNAUTHORIZED_ERROR: String =
        ErrorMessage::text(WebsocketError::AuthorizationError());
}
