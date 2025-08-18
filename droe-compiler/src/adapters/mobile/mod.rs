//! Mobile platform adapters for iOS and Android

pub mod ios;
pub mod android;

pub use ios::IOSAdapter;
pub use android::AndroidAdapter;