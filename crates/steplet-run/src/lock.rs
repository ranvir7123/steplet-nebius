//! One run at a time on this computer (phase 4 Task 3). `prepare` empties `/work`, so a second run would delete the
//! first one's working copy. The lock is a named Windows object, `Local\Steplet-run`: whoever made it first holds it
//! until every handle to it is closed, which Windows also does when a program ends or crashes. Only that the object
//! exists counts, so the lock can be dropped on any thread.

use std::io;
use std::ptr::null;

use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;

use crate::run::RunError;

const NAME: &str = "Local\\Steplet-run";

/// Held for the whole of one run.
#[derive(Debug)]
pub struct RunLock(HANDLE);

// SAFETY: the handle is used only to close it, which Windows allows from any thread.
unsafe impl Send for RunLock {}

impl RunLock {
    /// The lock, or `RunError::Busy` when another run (in this or another Steplet window) holds it.
    pub fn take() -> Result<RunLock, RunError> {
        let name: Vec<u16> = NAME.encode_utf16().chain([0]).collect();
        // SAFETY: `name` ends in a zero and outlives the call; no security attributes, not owned on creation.
        let handle = unsafe { CreateMutexW(null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(RunError::Lock(io::Error::last_os_error()));
        }
        // SAFETY: read right after the call that set it.
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            // SAFETY: `handle` was just opened and is closed once.
            unsafe { CloseHandle(handle) };
            return Err(RunError::Busy);
        }
        Ok(RunLock(handle))
    }
}

impl Drop for RunLock {
    fn drop(&mut self) {
        // SAFETY: the handle was opened by `take` and is closed once, here.
        unsafe { CloseHandle(self.0) };
    }
}
