use {
    crate::{Note, SurrealRecord},
    derive::PutId,
    serde::{Deserialize, Serialize},
};

#[cfg(feature = "server")]
use surrealdb::RecordId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, PutId)]
pub struct Knot {
    #[cfg(feature = "server")]
    pub id: RecordId,
    pub id_: Option<String>,
    pub intent: String,
    #[put_ids]
    pub notes: Vec<Note>,
    #[put_ids]
    pub knots: Vec<Knot>,
}
