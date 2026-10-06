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
        // IDF 6 uses Picolibc: C stdout's VFS descriptor need not be POSIX fd 1.
        // Rust println! writes to fd 1, which can be the optional USB-JTAG port
        // and fail before enumeration or when unplugged. ESP-IDF's logger uses
        // the configured C console and does not panic on an output error.
        esp_idf_sys::esp_log_write(
            esp_idf_sys::esp_log_level_t_ESP_LOG_INFO,
            c"p4desk_rust".as_ptr(),
            c"%.*s\n".as_ptr(),
            line.len as i32,
            line.bytes.as_ptr().cast::<std::ffi::c_char>(),
        );
    }
    #[cfg(not(target_os = "espidf"))]
    {
        use std::io::Write as _;
        let mut output = std::io::stdout().lock();
        let _ = output.write_all(&line.bytes[..line.len]);
        let _ = output.write_all(b"\n");
    }
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
}
