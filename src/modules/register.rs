use crate::*;

impl CommandHandler {
    pub fn register(&mut self) {
        #[cfg(feature = "modcore")]
        self.add_module(super::core::ModCore::new());
        #[cfg(feature = "modcoords")]
        self.add_module(super::coords::ModCoords::new());
    }
}
