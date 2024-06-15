use std::sync::OnceLock;

use tokio::sync::RwLock;

use crate::app_model::AppModel;

pub static APP_MODEL: OnceLock<RwLock<AppModel>> = OnceLock::new();

pub fn get_app_model() -> &'static RwLock<AppModel> {
    APP_MODEL.get_or_init(|| RwLock::new(AppModel::new()))
}
