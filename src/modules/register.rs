use crate::*;

impl CommandHandler {
    pub fn register(&mut self) {
        #[cfg(feature = "modcore")]
        self.add_module(super::core::ModCore::new());
    }
}
