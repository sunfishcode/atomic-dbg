#![no_std]

use core::cmp::min;
use core::fmt::{self, Write};
#[cfg(windows)]
use {
    io_lifetimes::BorrowedHandle,
    is_terminal::IsTerminal,
    core::ptr::null_mut,
    windows_sys::Win32::Foundation::{GetLastError, SetLastError, HANDLE},
    windows_sys::Win32::Storage::FileSystem::WriteFile,
    windows_sys::Win32::System::Console::STD_ERROR_HANDLE,
    windows_sys::Win32::System::Console::{GetStdHandle, WriteConsoleW},
};

// No specific size is documented, so just pick a number.
#[cfg(windows)]
const BUF_LEN: usize = 4096;

struct Writer {
    pos: usize,

    // `PIPE_BUF` is the biggest buffer we can write atomically.
    #[cfg(unix)]
    buf: [u8; rustix::io::PIPE_BUF],

    #[cfg(windows)]
    buf: [u8; BUF_LEN],

    // A buffer of UTF-16, for the Windows console.
    #[cfg(windows)]
    console_buf: [u16; BUF_LEN],

    #[cfg(windows)]
    console_pos: usize,

    #[cfg(feature = "errno")]
    #[cfg(unix)]
    saved_errno: errno::Errno,

    #[cfg(windows)]
    saved_error: u32,
}

impl Writer {
    fn new() -> Self {
        Self {
            pos: 0,

            #[cfg(unix)]
            buf: [0_u8; rustix::io::PIPE_BUF],

            #[cfg(windows)]
            buf: [0_u8; BUF_LEN],

            #[cfg(windows)]
            console_buf: [0_u16; BUF_LEN],

            #[cfg(windows)]
            console_pos: 0,

            #[cfg(feature = "errno")]
            #[cfg(unix)]
            saved_errno: errno::errno(),

            #[cfg(windows)]
            saved_error: unsafe { GetLastError() },
        }
    }

    fn flush(&mut self) {
        #[cfg(unix)]
        {
            self.flush_buf().unwrap();
        }

        #[cfg(windows)]
        {
            let stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
            if unsafe { BorrowedHandle::borrow_raw(stderr as _) }.is_terminal() {
                self.flush_console(stderr)
            } else {
                self.flush_buf()
            }
            .unwrap();
        }
    }

    fn flush_buf(&mut self) -> fmt::Result {
        let mut s = &self.buf[..self.pos];

        // Safety: Users using `dbg`/`eprintln`/`eprint` APIs should be aware
        // that these assume that the stderr file descriptor is open and valid
        // to write to.
        #[cfg(unix)]
        let stderr = unsafe { rustix::io::stderr() };

        #[cfg(windows)]
        let stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE) };

        while !s.is_empty() {
            #[cfg(unix)]
            match rustix::io::write(stderr, s) {
                Ok(n) => s = &s[n..],
                Err(rustix::io::Errno::INTR) => (),
                Err(_) => return Err(fmt::Error),
            }

            #[cfg(windows)]
            unsafe {
                let mut n = 0;
                if WriteFile(
                    stderr,
                    s.as_ptr().cast(),
                    min(s.len(), u32::MAX as usize) as u32,
                    &mut n,
                    null_mut(),
                ) == 0
                {
                    return Err(fmt::Error);
                }
                s = &s[n as usize..];
            }
        }

        self.pos = 0;
        Ok(())
    }

    #[cfg(windows)]
    fn flush_console(&mut self, handle: HANDLE) -> fmt::Result {
        // Write `self.console_buf` to the console.
        let mut s = &self.console_buf[..self.console_pos];
        while !s.is_empty() {
            unsafe {
                let mut n16 = 0;
                if WriteConsoleW(
                    handle,
                    s.as_ptr().cast(),
                    min(s.len(), u32::MAX as usize) as u32,
                    &mut n16,
                    null_mut(),
                ) == 0
                {
                    return Err(fmt::Error);
                }
                s = &s[n16 as usize..];
            }
        }

        self.console_pos = 0;
        Ok(())
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        #[cfg(feature = "errno")]
        #[cfg(unix)]
        errno::set_errno(self.saved_errno);

        #[cfg(windows)]
        unsafe {
            SetLastError(self.saved_error);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // On Windows, to write Unicode to the console, we need to use some
        // form of `WriteConsole` instead of `WriteFile`.
        #[cfg(windows)]
        {
            let stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
            if unsafe { BorrowedHandle::borrow_raw(stderr as _) }.is_terminal() {
                // If the stream switched on us, just fail.
                if self.pos != 0 {
                    return Err(fmt::Error);
                }

                // Transcode `s` to UTF-16 and write it to the console.
                for c in s.chars() {
                    if self.console_buf.len() - self.console_pos < 2 {
                        self.flush_console(stderr)?;
                    }
                    self.console_pos += c
                        .encode_utf16(&mut self.console_buf[self.console_pos..])
                        .len();
                }
                return Ok(());
            }

            // If the stream switched on us, just fail.
            if self.console_pos != 0 {
                return Err(fmt::Error);
            }
        }

        let mut bytes = s.as_bytes();
        while !bytes.is_empty() {
            let mut sink = &mut self.buf[self.pos..];
            if sink.is_empty() {
                self.flush_buf()?;
                sink = &mut self.buf;
            }

            let len = min(sink.len(), bytes.len());
            let (now, later) = bytes.split_at(len);
            sink[..len].copy_from_slice(now);

            self.pos += len;
            bytes = later;
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _eprint(args: fmt::Arguments<'_>) {
    let mut writer = Writer::new();
    writer.write_fmt(args).unwrap();
    writer.flush();
}

#[doc(hidden)]
pub fn _eprintln(args: fmt::Arguments<'_>) {
    let mut writer = Writer::new();
    writer.write_fmt(args).unwrap();
    writer.write_fmt(format_args!("\n")).unwrap();
    writer.flush();
}

#[doc(hidden)]
pub struct _Dbg(Writer);

#[doc(hidden)]
pub fn _dbg_start() -> _Dbg {
    _Dbg(Writer::new())
}

#[doc(hidden)]
pub fn _dbg_write(dbg: &mut _Dbg, file: &str, line: u32, name: &str, args: fmt::Arguments<'_>) {
    writeln!(&mut dbg.0, "[{}:{}] {} = {}", file, line, name, args).unwrap();
}

#[doc(hidden)]
pub fn _dbg_finish(mut dbg: _Dbg) {
    dbg.0.flush();
}

/// Prints to the standard error.
///
/// Similar to [`std::eprint`], except it:
///  - Writes atomically, up to the greatest length supported on the platform.
///  - Doesn't use locks (in userspace).
///  - Preserve libc's `errno` and Windows' last-error code value.
///
/// This allows it to be used to debug allocators, multi-threaded code,
/// synchronization routines, startup code, and more.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {
        $crate::_eprint(core::format_args!($($arg)*))
    };
}

/// Prints to the standard error, with a newline.
///
/// Similar to [`std::eprintln`], except it:
///  - Writes atomically, up to the greatest length supported on the platform.
///  - Doesn't use locks (in userspace).
///  - Preserve libc's `errno` and Windows' last-error code value.
///
/// This allows it to be used to debug allocators, multi-threaded code,
/// synchronization routines, startup code, and more.
#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::eprint!("\n")
    };
    ($($arg:tt)*) => {
        // TODO: Use `format_args_nl` when it's stabilized.
        $crate::_eprintln(core::format_args!($($arg)*))
    };
}

/// Prints and returns the value of a given expression for quick and dirty
/// debugging.
///
/// Similar to [`std::dbg`], except it:
///  - Writes atomically, up to the greatest length supported on the platform.
///  - Doesn't use locks (in userspace).
///  - Preserve libc's `errno` and Windows' last-error code value.
///
/// This allows it to be used to debug allocators, multi-threaded code,
/// synchronization routines, startup code, and more.
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::eprintln!("[{}:{}]", core::file!(), core::line!())
    };
    ($val:expr $(,)?) => {
        // Use the same `match` trick that `std` does.
        match $val {
            tmp => {
                $crate::eprintln!("[{}:{}] {} = {:#?}",
                    core::file!(), core::line!(), core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        let mut dbg = $crate::_dbg_start();

        // Use the same `match` trick that `std` does.
        ($(match $val {
            tmp => {
                $crate::_dbg_write(
                    &mut dbg,
                    core::file!(),
                    core::line!(),
                    core::stringify!($val),
                    core::format_args!("{:#?}", &tmp),
                );
                tmp
            }
        }),+);

        $crate::_dbg_finish(dbg);
    };
}
