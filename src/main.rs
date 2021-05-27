#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde;

use std::sync::Mutex;
use rocket::{Response, Rocket, State, http::Status, response::Redirect};
use rusqlite::{Connection, Error};

use rocket_contrib::json::{Json, JsonValue};

type DbConn = Mutex<Connection>;

#[derive(Serialize, Deserialize)]
struct GenerateRequest {
    link: String,
}

#[post("/api/generate", format = "json", data = "<message>")]
fn generate(db_conn: State<DbConn>, message: Json<GenerateRequest>) -> JsonValue {
    let link = message.link.as_str();
    let shortened_id = uuid::Uuid::new_v4().to_string();

    db_conn.lock().expect("Failed to lock").execute(
        "INSERT INTO urls(shortened_id, original_url) VALUES(?, ?)",
        &[shortened_id.as_str(), link]
    ).expect("Failed to insert link");
    rocket_contrib::json!({
        "id": shortened_id.as_str(),
        "status": "ok",
    })
}

#[get("/<id>")]
fn redirect(db_conn: State<DbConn>, id: String) -> Result<Redirect, Response> {
    let original_url: Result<String, rusqlite::Error> = db_conn.lock().expect("Failed to lock").query_row(
        "SELECT original_url FROM urls WHERE shortened_id = ?",
        &[id.as_str()],
        |row| row.get(0)
    );
    
    match original_url {
        Err(_) => Err(Response::build().status(Status::NotFound).finalize()),
        Ok(e) => Ok(Redirect::to(format!("{}", e.as_str()))),
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