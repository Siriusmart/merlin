use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::Config;

pub static mut COORDS_CONFIG: OnceLock<CoordsConfig> = OnceLock::new();

#[serde_inline_default]
#[derive(Serialize, Deserialize, Clone, DefaultFromSerde, Hash)]
pub struct CoordsConfig {
    #[serde_inline_default(100)]
    #[serde(rename = "prevent-add-radius")]
    pub prevent_add_radius: u64,
}

impl Config for CoordsConfig {
    const NAME: &'static str = "coords";
    const NOTE: &'static str = "Main config file for the coords module";
}

impl CoordsConfig {
    pub fn reload() {
        unsafe { COORDS_CONFIG = OnceLock::new() };
        Self::setup();
    }

    // pub fn write_to_config() {
    //     unsafe { COORDS_CONFIG.get() }.unwrap().smart_save();
    // }

    pub fn setup() {
        let _ = unsafe { COORDS_CONFIG.set(CoordsConfig::load()) };
    }
}
