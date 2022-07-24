/// Max length of our stack-string.
const STACK_STRING_SIZE: usize = 64;

/// Object to be able to write a string that's stored onto the stack.
pub struct StackString {
    pub buffer: [u8; STACK_STRING_SIZE],
    pub size: usize,
}
impl StackString {
    pub const STACK_STRING_SIZE: usize = STACK_STRING_SIZE;
    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr() as *const u8
    }
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn data<'a>(&'a self) -> &'a [u8] {
        &self.buffer[0..self.size]
    }
}

impl Default for StackString {
    fn default() -> Self {
        StackString {
            buffer: [0; Self::STACK_STRING_SIZE],
            size: 0,
        }
    }
}

use core::cmp::min;
use core::fmt::Error;
// Implement the Write trait for the stackstring.
impl core::fmt::Write for StackString {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for i in 0..min(s.len(), self.buffer.len() - self.size) {
            self.buffer[self.size] = s.as_bytes()[i];
            self.size += 1;
        }
        if self.size == self.buffer.len() {
            return Err(Error {});
        }
        Ok(())
    }
    // fn write_char(&mut self, c: char) -> Result { ... }
    // fn write_fmt(&mut self, args: Arguments<'_>) -> Result { ... }
}

pub use core::fmt;
/// Provide a println! macro similar to Rust does.
#[macro_export]
macro_rules! println {
    () => ($crate::io::print("\n"));
    ($($arg:tt)*) => ({
        let mut v: $crate::io::StackString = Default::default();
        core::fmt::write(&mut v, format_args!($($arg)*)).expect("Error occurred while trying to write in String");
        v.write_str("\n").expect("Shouldn't fail");
    })
}
