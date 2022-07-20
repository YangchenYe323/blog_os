//! This module provides abstraction over the VGA text buffer
//! to provides utilities of printing, etc.

use core::result::Result::Ok;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    /// Global writer instance that drives the VGA buffer.
    static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        /// we know that the buffer locates at memory-mapped address 0xb8000
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// Print to the global VGA buffer writer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Print ending with newline
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
  use core::fmt::Write;
  WRITER.lock().write_fmt(args).unwrap();
}

/// Represents the color recognized by VGA
/// Each color occupies at most the small four bits
/// of the underlieing u8
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Color {
  Black = 0,
  Blue = 1,
  Green = 2,
  Cyan = 3,
  Red = 4,
  Magenta = 5,
  Brown = 6,
  LightGray = 7,
  DarkGray = 8,
  LightBlue = 9,
  LightGreen = 10,
  LightCyan = 11,
  LightRed = 12,
  Pink = 13,
  Yellow = 14,
  White = 15,
}

/// ColorCode represents an entire color code byte for VGA,
/// whose layout is <background>|<foreground>.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
  /// Create a complete ColorCode from background and foreground colors.
  fn new(foreground: Color, background: Color) -> ColorCode {
    let color: u8 = (background as u8) << 4 | foreground as u8;
    Self(color)
  }
}

/// ScreenChar is the entire display unit in the VGA buffer. It contains two consecutive
/// bytes, where the first byte is an ASCII-encoded u8 and the second byte is a [ColorCode]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
  ascii_character: u8,
  color_code: ColorCode,
}

/// VGA text buffer has 25 rows and 80 coliumns
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// Represents a VGA text buffer
#[derive(Debug)]
#[repr(transparent)]
struct Buffer {
  /// Volatile wraps around [ScreenChar] transparently but make sure that
  /// the write access doesn't get optimized away.
  chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Writer represents a write handler to the VGA buffer.
/// It keeps track of current position and current color to make writing
/// to buffer easier.
#[derive(Debug)]
struct Writer {
  /// Current cursor position
  column_position: usize,
  /// Current color
  color_code: ColorCode,
  /// A static reference to the buffer area
  buffer: &'static mut Buffer,
}

impl Writer {
  /// Write a byte to the buffer
  fn write_byte(&mut self, byte: u8) {
    match byte {
      b'\n' => {
        self.new_line();
      }

      byte => {
        if self.column_position >= BUFFER_WIDTH {
          self.new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let color_code = self.color_code;

        self.buffer.chars[row][col].write(ScreenChar {
          ascii_character: byte,
          color_code,
        });

        self.column_position += 1;
      }
    }
  }

  /// Move every row up and clear the last row for future use.
  fn new_line(&mut self) {
    for row in 1..BUFFER_HEIGHT {
      for col in 0..BUFFER_WIDTH {
        let c = self.buffer.chars[row][col].read();
        self.buffer.chars[row - 1][col].write(c);
      }
    }
    self.clear_row(BUFFER_HEIGHT - 1);
    self.column_position = 0;
  }

  /// Fill blank to all the cells in row
  fn clear_row(&mut self, row: usize) {
    let blank = ScreenChar {
      ascii_character: b' ',
      color_code: self.color_code,
    };
    for col in 0..BUFFER_WIDTH {
      self.buffer.chars[row][col].write(blank);
    }
  }

  /// Write a string to the buffer
  fn write_string(&mut self, s: &str) {
    for byte in s.bytes() {
      match byte {
        // printable ASCII byte or newline
        0x20..=0x7e | b'\n' => self.write_byte(byte),
        // not part of printable ASCII range
        _ => self.write_byte(0xfe),
      }
    }
  }
}

impl core::fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    self.write_string(s);
    Ok(())
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use core::str::from_utf8;

  #[test_case]
  fn test_println_no_panic() {
    println!("shouldn't panic");
  }

  #[test_case]
  fn test_println_many() {
    for _ in 0..200 {
      println!("test_println_many output");
    }
  }

  #[test_case]
  fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
      let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
      assert_eq!(char::from(screen_char.ascii_character), c);
    }
  }

  #[test_case]
  fn test_long_print_wrap() {
    const LEN: usize = 2 * BUFFER_WIDTH as usize;
    let s: [u8; LEN] = [b'a'; LEN];
    // this should be wrapped and occupies two row
    print!("{}", from_utf8(&s).unwrap());

    for (i, c) in s.iter().enumerate() {
      let row_offset = i / BUFFER_WIDTH as usize;
      let col = i % BUFFER_WIDTH as usize;
      // serial_println!("{}, {}, {}", row_offset, col, i);

      let screen_char =
        WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2 + row_offset][col].read();
      assert_eq!(screen_char.ascii_character, *c);
    }
  }
}
