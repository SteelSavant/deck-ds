use native_db::Models;
use once_cell::sync::Lazy;

mod v1;

pub static MODELS: Lazy<native_db::Models> = Lazy::new(|| {
    let mut models = Models::new();

    todo!("define models");
    models
});
