use chrono::NaiveDateTime;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "server", derive(FromRow))]
pub struct VideoNote {
    pub id: String,
    pub location: String,
    pub created_at: NaiveDateTime,
}
