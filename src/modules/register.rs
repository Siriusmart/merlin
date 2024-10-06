use crate::*;

impl CommandHandler {
    pub fn register(&mut self) {
        self.add_module(ModCore::new());
    }
}
