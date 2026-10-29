//! One web request through WinHTTP, Windows' own web-request library (the Nebius plan, decision N1): Windows'
//! certificates, its proxy settings and its secure-connection code, and no new package. Only `https` is used, and plain
//! `http` to this computer for the tests' stand-in server. Redirects and cookies are off: an answer that points
//! elsewhere comes back as it is, and the key never follows it.

use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Networking::WinHttp::{
    ERROR_WINHTTP_TIMEOUT, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_ACCESS_TYPE_NO_PROXY, WINHTTP_DISABLE_COOKIES,
    WINHTTP_DISABLE_REDIRECTS, WINHTTP_FLAG_SECURE, WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2,
    WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3, WINHTTP_OPTION_DISABLE_FEATURE, WINHTTP_OPTION_SECURE_PROTOCOLS,
    WINHTTP_QUERY_CUSTOM, WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE, WinHttpCloseHandle, WinHttpConnect,
    WinHttpOpen, WinHttpOpenRequest, WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest,
    WinHttpSetOption, WinHttpSetTimeouts,
};

/// How long a request may take, and how big its answer may be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HttpLimits {
    pub connect: Duration,
    /// For the whole answer. Windows times each wait for data on its own, so an answer that trickles in can take up to
    /// twice this before it is cut off.
    pub reply: Duration,
    pub max_bytes: usize,
}

impl Default for HttpLimits {
    fn default() -> HttpLimits {
        HttpLimits { connect: Duration::from_secs(15), reply: Duration::from_secs(120), max_bytes: 4 << 20 }
    }
}

/// Where requests go: a secure address, or (tests only) plain `http` to this computer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub secure: bool,
    pub host: String,
    pub port: u16,
    /// Starts and ends with `/`.
    pub path: String,
}

impl Endpoint {
    /// `https://host[:port]/path/`, or `http://127.0.0.1:port/path/`; anything else is refused.
    pub fn parse(base: &str) -> Result<Endpoint, String> {
        let (secure, rest) = if let Some(rest) = base.strip_prefix("https://") {
            (true, rest)
        } else if let Some(rest) = base.strip_prefix("http://127.0.0.1:") {
            (false, rest)
        } else {
            return Err(format!("{base:?} is neither https nor this computer"));
        };
        let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
        let (host, port) = if secure {
            match authority.rsplit_once(':') {
                Some((h, p)) => (h, p.parse().map_err(|_| format!("{base:?} has a bad port"))?),
                None => (authority, 443),
            }
        } else {
            ("127.0.0.1", authority.parse().map_err(|_| format!("{base:?} has a bad port"))?)
        };
        let ok_host = !host.is_empty() && host.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
        if !ok_host {
            return Err(format!("{base:?} has a bad host name"));
        }
        let mut path = if path.is_empty() { "/".to_string() } else { path.to_string() };
        if !path.ends_with('/') {
            path.push('/');
        }
        Ok(Endpoint { secure, host: host.into(), port, path })
    }
}

/// An answer: its status, the `Retry-After` header if any, and its body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    pub retry_after: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpError {
    TooSlow,
    TooBig,
    Network(String),
}

/// A WinHTTP handle, closed when dropped. Handles are declared session, then connection, then request, so they are
/// closed in the reverse order, as WinHTTP expects.
struct Handle(*mut c_void);

impl Handle {
    fn new(h: *mut c_void) -> Result<Handle, HttpError> {
        if h.is_null() { Err(last_error()) } else { Ok(Handle(h)) }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: the handle came from WinHTTP and is closed once, here.
        unsafe { WinHttpCloseHandle(self.0) };
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

fn last_error() -> HttpError {
    // SAFETY: read right after the WinHTTP call that failed.
    let code = unsafe { GetLastError() };
    if code == ERROR_WINHTTP_TIMEOUT {
        return HttpError::TooSlow;
    }
    let what = match code {
        12007 => "the name was not found (no internet, or a wrong address)",
        12029 => "the connection could not be made",
        12030 | 12031 => "the connection was dropped",
        12038 | 12044 | 12045 | 12057 | 12169 | 12175 => "the secure connection failed (a certificate problem)",
        12152 => "the answer was not a web answer",
        _ => "a Windows web error",
    };
    HttpError::Network(format!("{what} (WinHTTP error {code})"))
}

fn check(ok: i32) -> Result<(), HttpError> {
    if ok == 0 { Err(last_error()) } else { Ok(()) }
}

/// Sends one request and reads the whole answer. `headers` are `Name: value` lines, each ending in `\r\n`; their copy
/// handed to Windows is wiped once sent, as it holds the key.
pub fn send(
    to: &Endpoint,
    method: &str,
    path: &str,
    headers: &str,
    body: &[u8],
    limits: &HttpLimits,
) -> Result<Answer, HttpError> {
    let started = Instant::now();
    let access = if to.secure { WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY } else { WINHTTP_ACCESS_TYPE_NO_PROXY };
    let agent = wide("Steplet");
    // SAFETY: `agent` ends in a zero and outlives the call; no proxy names, synchronous use.
    let session = Handle::new(unsafe { WinHttpOpen(agent.as_ptr(), access, null(), null(), 0) })?;
    let protocols: u32 = WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2 | WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3;
    // SAFETY: the option takes a u32, given by address with its size.
    let set = unsafe { WinHttpSetOption(session.0, WINHTTP_OPTION_SECURE_PROTOCOLS, (&raw const protocols).cast(), 4) };
    if set == 0 {
        let only12: u32 = WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2;
        // SAFETY: as above.
        check(unsafe { WinHttpSetOption(session.0, WINHTTP_OPTION_SECURE_PROTOCOLS, (&raw const only12).cast(), 4) })?;
    }
    let ms = |d: Duration| i32::try_from(d.as_millis()).unwrap_or(i32::MAX);
    // SAFETY: a valid session handle and plain numbers.
    check(unsafe {
        WinHttpSetTimeouts(session.0, ms(limits.connect), ms(limits.connect), ms(limits.reply), ms(limits.reply))
    })?;
    let host = wide(&to.host);
    // SAFETY: `host` ends in a zero and outlives the call.
    let connection = Handle::new(unsafe { WinHttpConnect(session.0, host.as_ptr(), to.port, 0) })?;
    let verb = wide(method);
    let object = wide(&format!("{}{}", to.path, path.trim_start_matches('/')));
    let flags = if to.secure { WINHTTP_FLAG_SECURE } else { 0 };
    // SAFETY: the strings end in zeros and outlive the call; default version, no referrer, any type accepted.
    let request = Handle::new(unsafe {
        WinHttpOpenRequest(connection.0, verb.as_ptr(), object.as_ptr(), null(), null(), null(), flags)
    })?;
    let off: u32 = WINHTTP_DISABLE_COOKIES | WINHTTP_DISABLE_REDIRECTS;
    // SAFETY: the option takes a u32, given by address with its size.
    check(unsafe { WinHttpSetOption(request.0, WINHTTP_OPTION_DISABLE_FEATURE, (&raw const off).cast(), 4) })?;

    let mut head = wide(headers);
    let length = u32::try_from(body.len()).map_err(|_| HttpError::TooBig)?;
    let data: *const c_void = if body.is_empty() { null() } else { body.as_ptr().cast() };
    // SAFETY: `head` ends in a zero (length -1 tells Windows so); `body` outlives the call; synchronous use.
    let sent = unsafe { WinHttpSendRequest(request.0, head.as_ptr(), u32::MAX, data, length, length, 0) };
    head.iter_mut().for_each(|c| *c = 0);
    check(sent)?;
    // SAFETY: a request that was sent.
    check(unsafe { WinHttpReceiveResponse(request.0, null_mut()) })?;

    let mut status: u32 = 0;
    let mut size: u32 = 4;
    // SAFETY: the status is asked for as a number, into a u32 of the size given.
    check(unsafe {
        WinHttpQueryHeaders(
            request.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            null(),
            (&raw mut status).cast(),
            &mut size,
            null_mut(),
        )
    })?;
    let retry_after = header(&request, "Retry-After");

    let mut answer = Vec::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        if started.elapsed() > limits.connect + limits.reply {
            return Err(HttpError::TooSlow);
        }
        let mut read: u32 = 0;
        // SAFETY: `buffer` holds as many bytes as asked for.
        check(unsafe { WinHttpReadData(request.0, buffer.as_mut_ptr().cast(), buffer.len() as u32, &mut read) })?;
        if read == 0 {
            break;
        }
        if answer.len() + read as usize > limits.max_bytes {
            return Err(HttpError::TooBig);
        }
        answer.extend_from_slice(&buffer[..read as usize]);
    }
    Ok(Answer { status: u16::try_from(status).unwrap_or(0), retry_after, body: answer })
}

/// A header of the answer, if present.
fn header(request: &Handle, name: &str) -> Option<String> {
    let name = wide(name);
    let mut buffer = vec![0u16; 256];
    let mut size = (buffer.len() * 2) as u32;
    // SAFETY: a custom header by name, into a buffer of the size given in bytes.
    let ok = unsafe {
        WinHttpQueryHeaders(
            request.0,
            WINHTTP_QUERY_CUSTOM,
            name.as_ptr(),
            buffer.as_mut_ptr().cast(),
            &mut size,
            null_mut(),
        )
    };
    if ok == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buffer[..size as usize / 2]))
}
