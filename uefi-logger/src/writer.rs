use super::Output;
use core::{fmt, str};

/// Struct which is used to implement the `fmt::Write` trait on a UEFI output protocol.
pub struct OutputWriter {
    output: &'static mut Output,
}

impl OutputWriter {
    pub fn new(output: &'static mut Output) -> Self {
        OutputWriter { output }
    }
}

impl fmt::Write for OutputWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Allocate a small buffer on the stack.
        const BUF_SIZE: usize = 128;
        // Add 1 extra character for the null terminator.
        let mut buf = [0u16; BUF_SIZE + 1];

        let mut i = 0;

        // This closure writes the local buffer to the output and resets the buffer.
        let mut flush_buffer = |buf: &mut [u16], i: &mut usize| {
            buf[*i] = 0;
            *i = 0;

            self.output.output_string(buf.as_ptr()).map_err(
                |_| fmt::Error,
            )
        };

        {
            // This closure converts a character to UCS-2 and adds it to the buffer,
            // flushing it as necessary.
            let mut add_char = |ch| {
                // UEFI only supports UCS-2 characters, not UTF-16,
                // so there are no multibyte characters.
                let ch = ch as u16;

                buf[i] = ch;

                i += 1;

                if i == BUF_SIZE {
                    flush_buffer(&mut buf, &mut i)
                } else {
                    Ok(())
                }
            };

            for ch in s.chars() {
                if ch == '\n' {
                    // Prepend an '\r'.
                    add_char('\r')?;
                }

                add_char(ch)?;
            }
        }

        // Flush whatever is left in the buffer.
        flush_buffer(&mut buf, &mut i)
    }
}
