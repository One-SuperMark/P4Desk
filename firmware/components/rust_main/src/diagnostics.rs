//! Bounded, best-effort device logs. A disconnected console must never reset the UI.
use std::fmt::{self, Write};

struct Line {
    bytes: [u8; 512],
    len: usize,
    truncated: bool,
}

impl Write for Line {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if self.truncated {
            return Ok(());
        }
        let mut count = text.len().min(self.bytes.len() - self.len);
        while !text.is_char_boundary(count) {
            count -= 1;
        }
        self.bytes[self.len..self.len + count].copy_from_slice(&text.as_bytes()[..count]);
        self.len += count;
        self.truncated = count < text.len();
        Ok(())
    }
}

pub(crate) fn log(args: fmt::Arguments<'_>) {
    let mut line = Line {
        bytes: [0; 512],
        len: 0,
        truncated: false,
    };
    let _ = line.write_fmt(args);
    #[cfg(target_os = "espidf")]
    unsafe {
        extern "C" {
            fn p4desk_log_diagnostic(data: *const u8, length: usize);
        }
        // Route through the C console: Rust fd 1 can refer to a disconnected
        // secondary USB descriptor on Picolibc. Formatting is bounded above.
        p4desk_log_diagnostic(line.bytes.as_ptr(), line.len);
    }
    #[cfg(not(target_os = "espidf"))]
    {
        let mut output = std::io::stdout().lock();
        write_buffer(&mut output, &line);
    }
}

#[cfg(not(target_os = "espidf"))]
fn write_buffer(output: &mut impl std::io::Write, line: &Line) {
    let _ = output.write_all(&line.bytes[..line.len]);
    let _ = output.write_all(b"\n");
}

macro_rules! diagnostic {
    ($($arg:tt)*) => { $crate::diagnostics::log(format_args!($($arg)*)) };
}
pub(crate) use diagnostic;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_utf8_log_is_bounded_and_keeps_a_valid_prefix() {
        let mut line = Line {
            bytes: [0; 512],
            len: 0,
            truncated: false,
        };
        write!(&mut line, "{}末尾", "中".repeat(200)).unwrap();
        assert_eq!(line.len, 510);
        assert!(line.truncated);
        assert_eq!(
            std::str::from_utf8(&line.bytes[..line.len]).unwrap(),
            "中".repeat(170)
        );
    }

    #[test]
    fn formatting_failure_is_not_a_panic() {
        struct Failed;
        impl fmt::Display for Failed {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                Err(fmt::Error)
            }
        }
        log(format_args!("diagnostic test {}", Failed));
    }

    #[test]
    fn disconnected_console_errors_are_best_effort() {
        struct FailedConsole(i32);
        impl std::io::Write for FailedConsole {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::from_raw_os_error(self.0))
            }
            fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
        }
        let mut line = Line { bytes: [0; 512], len: 0, truncated: false };
        write!(&mut line, "p4desk_session: save=ok apps=2").unwrap();
        for error in [2, 5, 9] { write_buffer(&mut FailedConsole(error), &line); }
        let mut output = Vec::new();
        write_buffer(&mut output, &line);
        assert_eq!(output, b"p4desk_session: save=ok apps=2\n");
    }

    #[test]
    fn exact_capacity_keeps_the_complete_prefix() {
        let mut line = Line { bytes: [0; 512], len: 0, truncated: false };
        write!(&mut line, "{}", "x".repeat(512)).unwrap();
        assert_eq!(line.len, 512);
        assert!(!line.truncated);
        write!(&mut line, "extra").unwrap();
        assert_eq!(line.len, 512);
        assert!(line.truncated);
    }
}
