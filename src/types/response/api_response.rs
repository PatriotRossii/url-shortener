use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    Request, Response,
};
use rocket_contrib::json::JsonValue;

#[derive(Debug)]
pub(crate) struct ApiResponse {
    status: Status,
    json: JsonValue,
}

impl ApiResponse {
    pub fn new(status: Status, json: JsonValue) -> Self {
        Self { status, json }
    }
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&request).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

pub(crate) fn internal_error() -> ApiResponse {
    ApiResponse::new(
        Status::InternalServerError,
        rocket_contrib::json!({
            "status": "internal server error"
        }),
    )
}
