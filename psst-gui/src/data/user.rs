use std::sync::Arc;

use druid::{Data, Lens};
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Data, Lens, Deserialize, Serialize)]
pub struct UserProfile {
    pub display_name: Arc<str>,
    pub email: Arc<str>,
    pub id: Arc<str>,
}

#[derive(Clone, Data, Lens, Deserialize, Serialize, Debug)]
pub struct PublicUser {
    pub display_name: Arc<str>,
    pub id: Arc<str>,
}
