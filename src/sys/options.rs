use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use super::Config;

pub static MASTER: OnceLock<MasterOptions> = OnceLock::new();

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde)]
pub struct MasterOptions {
    #[serde_inline_default(">".to_string())]
    pub prefix: String,
    #[serde_inline_default("CHANGE ME".to_string())]
    pub token: String,
}

impl Config for MasterOptions {
    const NAME: &'static str = "master";
    const NOTE: &'static str = "master config for merlin";
}

impl MasterOptions {
    pub fn setup() {
        let _ = MASTER.set(MasterOptions::load());
    }
}
