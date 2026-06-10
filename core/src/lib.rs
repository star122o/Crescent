pub mod minecraft;
pub mod accounts;

#[cfg(feature = "native-binding")]
uniffi::setup_scaffolding!();
