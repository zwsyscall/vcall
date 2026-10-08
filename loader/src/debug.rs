use windows::{
    Wdk::System::Threading::{NtSetInformationThread, THREADINFOCLASS},
    Win32::System::{
        Diagnostics::Debug::RemoveVectoredExceptionHandler, Threading::GetCurrentThread,
    },
};

// Flawless protection
pub fn no_strings_on_me() {
    unsafe {
        if NtSetInformationThread(
            GetCurrentThread(),
            THREADINFOCLASS(0x11),
            std::ptr::null(),
            0,
        )
        .is_err()
        {
            RemoveVectoredExceptionHandler(crate::veh::VEH_HANDLE);
        }
    }
}
