use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

use crate::{db::codec::rmp_serde_1_3::RmpSerde, settings_db::MonitorId};

#[native_db]
#[native_model(id = 1, version = 1, with = RmpSerde)]
#[derive(Serialize, Deserialize)]
pub struct DbDisplaySettings {
    #[primary_key]
    id: MonitorId,
}
