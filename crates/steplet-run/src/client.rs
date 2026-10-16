//! Who is at the other end of the run's pipe, as Windows says (plan Task 8): the program, its Windows account and
//! sign-in session, its integrity level, and whether its token is restricted or in an app container. Every Windows
//! call Steplet makes about a client (including waiting for it to read its answer), and the pipe's permission list,
//! are in this file.

use std::io;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, LocalFree};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{
    GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, IsTokenRestricted, RevertToSelf,
    SECURITY_ATTRIBUTES, SecurityIdentification, TOKEN_INFORMATION_CLASS, TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
    TOKEN_STATISTICS, TOKEN_USER, TokenImpersonation, TokenIntegrityLevel, TokenIsAppContainer, TokenStatistics,
    TokenUser,
};
use windows_sys::Win32::Storage::FileSystem::FlushFileBuffers;
use windows_sys::Win32::System::Pipes::{GetNamedPipeClientProcessId, ImpersonateNamedPipeClient};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, OpenProcess, OpenProcessToken, OpenThreadToken,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};

/// Windows' number for medium integrity, the level of a program a person starts normally.
pub(crate) const MEDIUM_INTEGRITY: u32 = 0x2000;

/// A program connected to the run's pipe, or Steplet itself, as Windows describes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    /// The process id.
    pub pid: u32,
    /// The program, as Windows names it.
    pub program: String,
    /// The Windows account, as its security id (`S-1-5-21-…`).
    pub user: String,
    /// The sign-in session the program belongs to (Windows' logon session id).
    pub logon_session: u64,
    /// The integrity level's number: 4096 is low, 8192 medium, 12288 high.
    pub integrity: u32,
    /// The token is restricted: Windows checks every access again against a shorter list.
    pub restricted: bool,
    /// The program runs in an app container (a Windows sandbox).
    pub app_container: bool,
    /// For a pipe client: it lets Steplet see who it is and nothing more (it opened the pipe at identification
    /// level, so Steplet cannot act as it). Always false for Steplet itself. Recorded, not required: it protects the
    /// client from Steplet, not Steplet from the client.
    pub identify_only: bool,
}

impl Client {
    /// This program itself, to compare clients with.
    pub fn this_program() -> io::Result<Client> {
        let mut token = null_mut();
        // SAFETY: the current process's pseudo handle needs no closing; `token` receives a new handle.
        check(unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) })?;
        let token = Handle(token);
        // SAFETY: as above.
        let program = program_of(unsafe { GetCurrentProcess() })?;
        described(std::process::id(), program, &token)
    }

    /// The client at the other end of `pipe`, a connected server end that has already read from it (Windows lets a
    /// server see its client's token only after the client has written).
    pub(crate) fn of_pipe(pipe: HANDLE) -> io::Result<Client> {
        let mut pid = 0;
        // SAFETY: `pipe` is an open server end of a pipe.
        check(unsafe { GetNamedPipeClientProcessId(pipe, &mut pid) })?;
        // SAFETY: opening a process by id for querying only; the handle is closed by `Handle`.
        let process = Handle(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) });
        if process.0.is_null() {
            return Err(io::Error::last_os_error());
        }
        let program = program_of(process.0)?;
        let token = client_token(pipe)?;
        described(pid, program, &token)
    }
}

/// An open Windows handle, closed when dropped.
struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the handle is open and owned by this value.
            unsafe { CloseHandle(self.0) };
        }
    }
}

fn check(ok: i32) -> io::Result<()> {
    if ok == 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

fn program_of(process: HANDLE) -> io::Result<String> {
    let mut buf = vec![0u16; 32_768];
    let mut len = buf.len() as u32;
    // SAFETY: `buf` holds `len` wide characters; flags 0 asks for the usual Windows path.
    check(unsafe { QueryFullProcessImageNameW(process, 0, buf.as_mut_ptr(), &mut len) })?;
    Ok(String::from_utf16_lossy(&buf[..len as usize]))
}

/// The token the client connected with. This thread takes on the client's token only for the moment it takes to
/// open it, and gives it back before anything else runs.
fn client_token(pipe: HANDLE) -> io::Result<Handle> {
    // SAFETY: `pipe` is a connected server end that has read from its client.
    check(unsafe { ImpersonateNamedPipeClient(pipe) })?;
    let mut token = null_mut();
    // SAFETY: this thread's pseudo handle; opened with Steplet's own rights (`OpenAsSelf`), for querying only.
    let opened = check(unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut token) });
    // SAFETY: ends the impersonation started above.
    if unsafe { RevertToSelf() } == 0 {
        // Running on as someone else would be worse than stopping.
        std::process::abort();
    }
    opened.map(|()| Handle(token))
}

fn described(pid: u32, program: String, token: &Handle) -> io::Result<Client> {
    let user = info(token, TokenUser)?;
    // SAFETY: the buffer holds a TOKEN_USER, whose SID points inside the same buffer.
    let user = sid_text(unsafe { (*user.as_ptr().cast::<TOKEN_USER>()).User.Sid })?;
    let stats = info(token, TokenStatistics)?;
    // SAFETY: the buffer holds a TOKEN_STATISTICS.
    let stats = unsafe { &*stats.as_ptr().cast::<TOKEN_STATISTICS>() };
    let session = stats.AuthenticationId;
    let label = info(token, TokenIntegrityLevel)?;
    // SAFETY: the buffer holds a TOKEN_MANDATORY_LABEL, whose SID points inside the same buffer; an integrity SID
    // has one sub-authority, the level.
    let integrity = unsafe {
        let sid = (*label.as_ptr().cast::<TOKEN_MANDATORY_LABEL>()).Label.Sid;
        let count = *GetSidSubAuthorityCount(sid);
        *GetSidSubAuthority(sid, u32::from(count.saturating_sub(1)))
    };
    let app_container = info(token, TokenIsAppContainer)?;
    Ok(Client {
        pid,
        program,
        user,
        logon_session: (u64::from(session.HighPart as u32) << 32) | u64::from(session.LowPart),
        integrity,
        // SAFETY: `token` is open with query rights.
        restricted: unsafe { IsTokenRestricted(token.0) } != 0,
        app_container: app_container.first().is_some_and(|&v| v as u32 != 0),
        identify_only: stats.TokenType == TokenImpersonation && stats.ImpersonationLevel == SecurityIdentification,
    })
}

/// One kind of token information, in a buffer aligned for any of Windows' token structures.
fn info(token: &Handle, class: TOKEN_INFORMATION_CLASS) -> io::Result<Vec<u64>> {
    let mut len = 0;
    // SAFETY: asks only for the size.
    unsafe { GetTokenInformation(token.0, class, null_mut(), 0, &mut len) };
    let mut buf = vec![0u64; (len as usize).div_ceil(8).max(1)];
    // SAFETY: `buf` holds at least `len` bytes.
    check(unsafe { GetTokenInformation(token.0, class, buf.as_mut_ptr().cast(), len, &mut len) })?;
    Ok(buf)
}

fn sid_text(sid: *mut core::ffi::c_void) -> io::Result<String> {
    let mut text = null_mut();
    // SAFETY: `sid` is a valid SID; Windows allocates `text`, freed below.
    check(unsafe { ConvertSidToStringSidW(sid, &mut text) })?;
    // SAFETY: `text` is a NUL-terminated wide string from Windows.
    let s = unsafe {
        let len = (0..).take_while(|&i| *text.add(i) != 0).count();
        String::from_utf16_lossy(std::slice::from_raw_parts(text, len))
    };
    // SAFETY: allocated by ConvertSidToStringSidW.
    unsafe { LocalFree(text.cast()) };
    Ok(s)
}

/// Waits until the client at the other end of `pipe` (a server end's handle) has read everything written to it.
/// Ends with an error if the client closes or is cut off first.
pub(crate) fn wait_until_read(pipe: usize) -> io::Result<()> {
    // SAFETY: the caller keeps the pipe's handle open until this returns.
    check(unsafe { FlushFileBuffers(pipe as HANDLE) })
}

/// The pipe's permission list: only `user` (Steplet's own Windows account) may open it or add an end to it, and
/// programs below medium integrity may neither read nor write it. Windows' default list would let everyone read.
pub(crate) struct Permissions(Vec<u64>);

impl Permissions {
    pub(crate) fn only(user: &str) -> io::Result<Permissions> {
        let sddl: Vec<u16> = format!("D:P(A;;GA;;;{user})S:(ML;;NWNR;;;ME)").encode_utf16().chain([0]).collect();
        let (mut sd, mut len) = (null_mut(), 0);
        // SAFETY: `sddl` is NUL-terminated; Windows allocates `sd`, copied and freed below.
        check(unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl.as_ptr(), SDDL_REVISION_1, &mut sd, &mut len)
        })?;
        let mut copy = vec![0u64; (len as usize).div_ceil(8)];
        // SAFETY: `sd` holds `len` bytes of a self-relative security descriptor, which can be copied as bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(sd.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), len as usize);
            LocalFree(sd);
        }
        Ok(Permissions(copy))
    }

    /// Security attributes pointing at this list, for one call that creates an end of the pipe.
    pub(crate) fn attributes(&mut self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0.as_mut_ptr().cast(),
            bInheritHandle: 0,
        }
    }
}
