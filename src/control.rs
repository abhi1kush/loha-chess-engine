//! Search controls. The GUI control watches stdin; the offline control does not.

use crate::input::{self, poll_input};
use crate::ports::{SearchControl, SearchInfo};

pub struct GuiControl;

impl SearchControl for GuiControl {
    fn now_ms(&self) -> u64 {
        input::now_ms()
    }

    fn poll(&mut self, info: &mut SearchInfo) {
        poll_input(info);
    }
}

pub struct OfflineControl;

impl SearchControl for OfflineControl {
    fn now_ms(&self) -> u64 {
        input::now_ms()
    }

    fn poll(&mut self, _info: &mut SearchInfo) {}
}
