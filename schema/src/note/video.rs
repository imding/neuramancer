use chrono::NaiveDateTime;
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VideoNote {
    pub id: Uuid,
    pub location: String,
    pub created_at: NaiveDateTime,
}

