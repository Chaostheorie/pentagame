/*
db.models - Should contain struct models of the database schema defined
            in db.schema. Might also contain Insertable versions of those
            models.

Note:
In general all struct fields are kept public.
See https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html for
a comprehensive mapping of PG, rust and diesel types
*/

use super::schema::*;
use diesel::{Associations, Identifiable, Queryable};
use serde::Serialize;
use uuid::Uuid;

#[derive(Identifiable, Serialize, Queryable, Clone, PartialEq, Debug)]
pub struct Game {
    pub id: i32,
    pub name: String,
    pub state: i16,
    pub description: Option<String>,
}

#[derive(Identifiable, Serialize, Queryable, Clone, PartialEq, Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub password: String, // Argon2 hash
}

#[derive(Identifiable, Associations, Queryable, PartialEq, Debug)]
#[table_name = "user_games"]
#[belongs_to(User)]
#[belongs_to(Game)]
pub struct UserGame {
    pub id: i32,
    pub user_id: Uuid,
    pub game_id: i32,
}
