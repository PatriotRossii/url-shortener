#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;

use rocket::{
    http::{ContentType, Status},
    response::{self, Redirect, Responder},
    Request, Response, Rocket, State,
};
use rusqlite::Connection;
use std::sync::Mutex;

use rocket_contrib::json::{Json, JsonValue};

type DbConn = Mutex<Connection>;

#[derive(Serialize, Deserialize)]
struct GenerateRequest {
    link: String,
}

#[derive(Debug)]
struct ApiResponse {
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

fn internal_error() -> ApiResponse {
    ApiResponse::new(
        Status::InternalServerError,
        rocket_contrib::json!({
            "status": "internal server error"
        }),
    )
}

#[post("/api/generate", format = "json", data = "<message>")]
fn generate(
    db_conn: State<DbConn>,
    message: Json<GenerateRequest>,
) -> Result<ApiResponse, ApiResponse> {
    let link = message.link.as_str();
    let shortened_id = uuid::Uuid::new_v4().to_string();

    db_conn
        .lock()
        .map_err(|_| internal_error())?
        .execute(
            "INSERT INTO urls(shortened_id, original_url) VALUES(?, ?)",
            &[shortened_id.as_str(), link],
        )
        .map_err(|_| internal_error())?;
    Ok(ApiResponse::new(
        Status::Ok,
        rocket_contrib::json!({
        "id": shortened_id.as_str(),
        "status": "ok",
        }),
    ))
}

#[get("/<id>")]
fn redirect(db_conn: State<DbConn>, id: String) -> Result<Redirect, ApiResponse> {
    let original_url: Result<String, rusqlite::Error> =
        db_conn.lock().map_err(|_| internal_error())?.query_row(
            "SELECT original_url FROM urls WHERE shortened_id = ?",
            &[id.as_str()],
            |row| row.get(0),
        );

    match original_url {
        Err(_) => Err(ApiResponse::new(
            Status::NotFound,
            rocket_contrib::json!({
                "status": "not found"
            }),
        )),
        Ok(e) => Ok(Redirect::to(e)),
    }
}

fn rocket() -> Rocket {
    // Open a new in-memory SQLite database.
    let conn = Connection::open("database/database.sqlite3").expect("Failed to open database");

    // Have Rocket manage the database pool.
    rocket::ignite()
        .manage(Mutex::new(conn))
        .mount("/", routes![generate, redirect])
}

fn main() {
    rocket().launch();
}
