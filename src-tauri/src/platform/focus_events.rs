use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationFocusChangedEventHandler,
    IUIAutomationFocusChangedEventHandler_Impl,
};
use windows_core::{implement, Ref};

#[implement(IUIAutomationFocusChangedEventHandler)]
pub struct FocusEvents;

impl IUIAutomationFocusChangedEventHandler_Impl for FocusEvents_Impl {
    fn HandleFocusChangedEvent(
        &self,
        _: Ref<'_, IUIAutomationElement>,
    ) -> windows_core::Result<()> {
        // UIA calls on its own thread. Keep all inspection on the existing watcher.
        super::watcher::queue_refresh();
        Ok(())
    }
}
