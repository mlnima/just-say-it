use dictation_core::session::Delivery;
use std::sync::{atomic::AtomicU64, Arc};

#[derive(Clone, Default)]
pub struct OverlayHandle;
impl OverlayHandle {
    pub fn set_mode(&self, _: u8) {}
    pub fn stop(&self) {}
}

#[derive(Clone, Default)]
pub struct Target;
impl Target {
    pub fn known(&self) -> bool {
        false
    }
}

pub struct Focus;
impl Focus {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
    pub fn inspect(&self) -> Target {
        Target
    }
    pub fn insert(&self, _: &Target, _: &str, _: &AtomicU64, _: u64) -> Delivery {
        Delivery::Rejected
    }
}

pub fn start(runtime: crate::runtime::Runtime) {
    runtime.update(|s| {
        s.input_detection_supported = false;
        s.target_status =
            "Native input detection and delivery are not implemented on this platform.".into();
    });
}
