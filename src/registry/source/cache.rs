use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CacheMeta {
    pub(crate) last_update: DateTime<Utc>,
}