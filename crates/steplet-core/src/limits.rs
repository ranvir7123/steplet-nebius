//! Size limits (decision row 12). Starting values, to tune on the test corpus in phase 2. Over a limit is red.

pub const MAX_FILE_BYTES: usize = 256 * 1024;
pub const MAX_HEADER_BYTES: usize = 32 * 1024;
pub const MAX_COMMANDS: usize = 25;
pub const MAX_DOMAINS: usize = 15;
pub const MAX_FOLDERS_PER_LIST: usize = 10;
pub const MAX_STEPS: usize = 30;
/// "Any command under 300 characters": 299 is the longest allowed. Characters are Unicode scalar values.
pub const MAX_COMMAND_CHARS: usize = 299;
/// Not in row 12. A starting value for the `never` list, equal to the command limit.
pub const MAX_NEVER: usize = 25;
/// The furthest an `expires` date may be from the day of signing (row 19).
pub const MAX_EXPIRY_DAYS: i64 = 365;
/// The expiry `steplet-sign` suggests (row 19).
pub const DEFAULT_EXPIRY_DAYS: i64 = 14;
