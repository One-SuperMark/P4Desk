//! Diagnostic output is best effort: a missing console must not reboot the Pad.
use std::fmt;
#[cfg(not(target_os = "espidf"))]
use std::io::{self, Write};

#[cfg(not(target_os = "espidf"))]
pub(crate) fn log(args: fmt::Arguments<'_>) {
    write_line(&mut io::stdout().lock(), args);
}

#[cfg(not(target_os = "espidf"))]
fn write_line(output: &mut impl Write, args: fmt::Arguments<'_>) {
    // println! panics on a console IO error. In firmware panic=abort turns that
    // into a reboot loop, including when the error is in a startup diagnostic.
    // Do not report this error through the same failing console or retry here.
    let _ = writeln!(output, "{args}");
}

#[cfg(target_os = "espidf")]
pub(crate) fn log(args: fmt::Arguments<'_>) {
    extern "C" {
        fn p4desk_log_diagnostic(data: *const u8, length: usize);
    }
    let mut line = DiagnosticLine::new();
    let _ = fmt::write(&mut line, args);
    // IDF's UART + secondary USB console opens internal descriptors before its
    // stdio streams. Rust's fixed fd 1 can therefore address disconnected USB
    // instead of C stdout. Use IDF's C log route, without heap output or panic.
    unsafe { p4desk_log_diagnostic(line.bytes.as_ptr(), line.len) };
}

#[cfg(any(target_os = "espidf", test))]
struct DiagnosticLine {
    bytes: [u8; 512],
    len: usize,
}

#[cfg(any(target_os = "espidf", test))]
impl DiagnosticLine {
    fn new() -> Self {
        Self {
            bytes: [0; 512],
            len: 0,
        }
    }
}

#[cfg(any(target_os = "espidf", test))]
impl fmt::Write for DiagnosticLine {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let mut count = value.len().min(self.bytes.len() - self.len);
        while !value.is_char_boundary(count) {
            count -= 1;
        }
        self.bytes[self.len..self.len + count].copy_from_slice(&value.as_bytes()[..count]);
        self.len += count;
        if count == value.len() {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}

macro_rules! diagnostic {
    ($($arg:tt)*) => {
        $crate::diagnostics::log(format_args!($($arg)*))
    };
}
pub(crate) use diagnostic;

#[cfg(test)]
mod tests {
    use super::*;

    struct FailedConsole {
        error: i32,
    }

    impl Write for FailedConsole {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::from_raw_os_error(self.error))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn missing_console_does_not_panic_on_startup_or_checkpoint_logs() {
        // ENOENT=2 is the error captured from the rebooting board. Also cover
        // EIO=5 and EBADF=9, without relying on a connected host console.
        for error in [2, 5, 9] {
            let mut output = FailedConsole { error };
            write_line(&mut output, format_args!("p4desk_session: restore=empty"));
            write_line(
                &mut output,
                format_args!("p4desk_session: save={} apps={}", "ok", 2),
            );
        }
    }

    #[test]
    fn diagnostic_keeps_format_and_newline() {
        let mut output = Vec::new();
        write_line(
            &mut output,
            format_args!("p4desk_session: writer={}", "ready"),
        );
        assert_eq!(output, b"p4desk_session: writer=ready\n");
    }

    #[test]
    fn full_output_buffer_does_not_panic() {
        let mut storage = [0u8; 4];
        let mut output = storage.as_mut_slice();
        write_line(&mut output, format_args!("p4desk_session: restore=empty"));
        assert_eq!(&storage, b"p4de");
    }

    #[test]
    fn firmware_line_formats_counts_without_heap_output() {
        let mut line = DiagnosticLine::new();
        fmt::write(
            &mut line,
            format_args!("p4desk_session: save={} apps={}", "ok", 2),
        )
        .unwrap();
        assert_eq!(&line.bytes[..line.len], b"p4desk_session: save=ok apps=2");
    }

    #[test]
    fn firmware_line_bounds_output_at_utf8_boundary() {
        let mut line = DiagnosticLine::new();
        assert!(fmt::write(&mut line, format_args!("{}完成", "x".repeat(511))).is_err());
        assert_eq!(line.len, 511);
        assert!(std::str::from_utf8(&line.bytes[..line.len]).is_ok());
        assert_eq!(line.bytes[511], 0);
    }

    #[test]
    fn firmware_line_accepts_exact_capacity_and_rejects_more() {
        let mut line = DiagnosticLine::new();
        fmt::write(&mut line, format_args!("{}", "x".repeat(512))).unwrap();
        assert_eq!(line.len, 512);
        assert!(fmt::write(&mut line, format_args!("extra")).is_err());
        assert_eq!(line.len, 512);
    }
}
