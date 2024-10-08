use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use super::Config;

pub static mut MASTER: OnceLock<MasterOptions> = OnceLock::new();

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Hash)]
pub struct MasterOptions {
    #[serde_inline_default(".".to_string())]
    pub prefix: String,
    #[serde_inline_default("CHANGE ME".to_string())]
    pub token: String,
}

impl Config for MasterOptions {
    const NAME: &'static str = "master";
    const NOTE: &'static str = "master config for merlin";
}

impl MasterOptions {
    pub fn reload() {
        unsafe { MASTER = OnceLock::new() };
        Self::setup();
    }

    pub fn write_to_config() {
        unsafe { MASTER.get() }.unwrap().smart_save();
    }

    pub fn setup() {
        let _ = unsafe { MASTER.set(MasterOptions::load()) };
    }
}
