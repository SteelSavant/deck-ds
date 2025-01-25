use native_db::Models;
use once_cell::sync::Lazy;

mod v1;

pub type DbMonitorDisplaySettings = v1::DbMonitorDisplaySettings;
pub type DbSystemDisplay = v1::DbSystemDisplay;
pub type DbEmbeddedDisplaySettings = v1::DbEmbeddedDisplaySettings;

pub static MODELS: Lazy<native_db::Models> = Lazy::new(|| {
    let mut models = Models::new();

    models
        .define::<v1::DbMonitorDisplaySettings>()
        .expect("failed to define DbMonitorDisplaySettings v1");

    models
        .define::<v1::DbEmbeddedDisplaySettings>()
        .expect("failed to define DbMonitorDisplaySettings v1");

    models
});
