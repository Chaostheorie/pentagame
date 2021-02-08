
// imports
use super::helper::log_error;
use super::routes::UserResponse;
use super::templates;
use actix_web::{
    dev::HttpResponseBuilder, error::BlockingError, error::ResponseError, http::header,
    http::StatusCode, Error as WebError, HttpResponse,
};
use askama_actix::TemplateIntoResponse;
use derive_more::Display;

/*
UserError:
    displays user when something is not e.g. available as html for GET Requests
    returns template error.html

    Errors:
    ValidationError: Only returned for non-auth queries as everything else is API (POST) based
    InternalError: Something went really, really wrong but was gracefully caught
*/
#[derive(Debug, Display)]
pub enum UserError {
    InternalError(String),
    ValidationError(String),
    BlockingError(String),
    NotFoundError(),
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        let response = match self {
            UserError::InternalError(e) => {
                log_error("Internal Panic", e.clone());
                templates::ErrorTemplate {
                    message: self.into(),
                    code: self.status_code().as_u16(),
                    id: None,
                }
                .into_response()
            }
            _ => templates::ErrorTemplate {
                message: self.into(),
                code: self.status_code().as_u16(),
                id: None,
            }
            .into_response(),
        };

        match response {
            Ok(resp) => resp,
            Err(why) => {
                log_error("[User Error - Construction Failed]", why.to_string());

                HttpResponseBuilder::new(self.status_code())
                    .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(self.to_string())
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl UserError {
    pub fn wrap_template(res: Result<HttpResponse, WebError>) -> UserResponse {
        match res {
            Ok(response) => Ok(response),
            Err(why) => {
                log_error("Internal Error", why.to_string());
                Err(UserError::InternalError(
                    "Failed to render requested template".to_owned(),
                ))
            }
        }
    }
}

// convert WebErrors to UserErrors
impl From<WebError> for UserError {
    fn from(error: WebError) -> UserError {
        UserError::InternalError(error.to_string())
    }
}

// Convert Thread BlockingErrors to UserErrors
impl From<BlockingError<WebError>> for UserError {
    fn from(error: BlockingError<WebError>) -> UserError {
        match error {
            BlockingError::Error(web_error) => UserError::InternalError(web_error.to_string()),
            BlockingError::Canceled => UserError::BlockingError("Thread blocking error".into()),
        }
    }
}

impl From<BlockingError<UserError>> for UserError {
    fn from(error: BlockingError<UserError>) -> UserError {
        match error {
            BlockingError::Error(user_error) => user_error,
            BlockingError::Canceled => UserError::BlockingError("Thread blocking error".into()),
        }
    }
}

// String casting for UserErrors
impl From<UserError> for String {
    fn from(error: UserError) -> String {
        match error {
            UserError::NotFoundError { .. } => "Didn't found the requested resource".to_owned(),
            UserError::InternalError(message) => message,
            UserError::ValidationError(message) => message,
            UserError::BlockingError(message) => message,
        }
    }
}

impl<'a> From<&'a UserError> for String {
    fn from(error: &'a UserError) -> String {
        match error {
            UserError::NotFoundError { .. } => "Didn't found the requested resource".to_owned(),
            UserError::InternalError(message) => message.to_owned(),
            UserError::ValidationError(message) => message.to_owned(),
            UserError::BlockingError(message) => message.to_owned(),
        }
    }
}
