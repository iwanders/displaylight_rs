pub struct WriteOverrun();
// pub struct ReadOverrun();

/*
    if read_pos == write_pos, no data.
    write_pos denotes where where we are going to write.
    read_pos denotes up to where we have read.
*/

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
// https://doc.rust-lang.org/nomicon/atomics.html#relaxed
// https://doc.rust-lang.org/core/sync/atomic/struct.AtomicUsize.html#method.load
// Read with relaxed.
// some_var.load(Ordering::Relaxed)
// Store with release

/// Simple SpScQueue, holds up to N - 1 elements.
pub struct SpScRingbuffer<T, const N: usize> {
    array: [UnsafeCell<MaybeUninit<T>>; N],
    read_pos: AtomicUsize,
    write_pos: AtomicUsize,
}

impl<T, const N: usize> SpScRingbuffer<T, N> {
    const VAL: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());
    // type Writer = Writer<'a, T, N>; // nope; https://github.com/rust-lang/rust/issues/8995

    pub const fn new() -> Self {
        // Odd workaround for E0277
        SpScRingbuffer::<T, N> {
            array: [Self::VAL; N],
            read_pos: AtomicUsize::new(0),
            write_pos: AtomicUsize::new(0),
        }
    }

    #[cfg(test)]
    pub fn set_read_pos(&mut self, v: usize) {
        self.read_pos = AtomicUsize::new(v);
    }

    #[cfg(test)]
    pub fn set_write_pos(&mut self, v: usize) {
        self.write_pos = AtomicUsize::new(v);
    }

    pub fn is_empty(&self) -> bool {
        self.read_pos.load(Ordering::Relaxed) == self.write_pos.load(Ordering::Relaxed)
    }

    pub fn is_full(&self) -> bool {
        ((self.write_pos.load(Ordering::Relaxed) + 1) % N) == self.read_pos.load(Ordering::Relaxed)
    }

    /// Read a value out of the SpScQueue, advancing the read position.
    pub fn read_value(&mut self) -> Option<T> {
        unsafe { self.read_value_unsafe() }
    }

    /// Peek at the next readable value, without advancing the write position.
    pub fn peek_value(&self) -> Option<&T> {
        unsafe { self.peek_value_unsafe() }
    }

    // Private worker functiont that can use interior mutability to allow reading values on a const
    // version of self without having to do const casts.
    unsafe fn read_value_unsafe(&self) -> Option<T> {
        // check if there's something to return
        if self.is_empty() {
            return None;
        }
        // Get the read position.
        let r_pos = self.read_pos.load(Ordering::Relaxed);

        // Rip the value out of the array
        // only unsafe statement in this function.
        let v = self.array[r_pos].get().read().assume_init();

        // Update the new read position.
        self.read_pos.store((r_pos + 1) % N, Ordering::Release);
        Some(v)
    }

    // Private worker functiont that can use interior mutability to allow reading values on a const
    // version of self without having to do const casts.
    unsafe fn peek_value_unsafe(&self) -> Option<&T> {
        // check if there's something to return
        if self.is_empty() {
            return None;
        }
        // Get the read position.
        let r_pos = self.read_pos.load(Ordering::Relaxed);

        // Get a reference to the value.
        let v = self.array[r_pos].get().as_ref().unwrap().assume_init_ref();

        // Return it.
        Some(v)
    }

    /// Write a value to the SpScQueue, advancing the write position. Returns an error if the
    /// buffer is full.
    pub fn write_value(&mut self, value: T) -> Result<(), WriteOverrun> {
        unsafe { self.write_value_unsafe(value) }
    }

    // Write worker function that uses interior mutability.
    unsafe fn write_value_unsafe(&self, value: T) -> Result<(), WriteOverrun> {
        // Check if we can write
        if self.is_full() {
            return Err(WriteOverrun());
        }

        // Obtain the write position.
        let w_pos = self.write_pos.load(Ordering::Relaxed);

        // Insert the value into the array.
        let location = self.array[w_pos].get();

        // The following is safe, because r_pos can never be w_pos.
        // only unsafe statement in this function.
        location.write(MaybeUninit::new(value));

        // Advance the write pointer.
        self.write_pos.store((w_pos + 1) % N, Ordering::Release);
        Ok(())
    }

    pub fn split<'a>(&'a mut self) -> (Reader<'a, T, N>, Writer<'a, T, N>) {
        (Reader::new(self), Writer::new(self))
    }
}

pub struct Writer<'a, T, const N: usize> {
    buffer: &'a SpScRingbuffer<T, { N }>,
}

impl<'a, T, const N: usize> Writer<'a, T, { N }> {
    fn new(buffer: &'a SpScRingbuffer<T, { N }>) -> Self {
        Self { buffer }
    }

    pub fn write_value(&mut self, value: T) -> Result<(), WriteOverrun> {
        unsafe { self.buffer.write_value_unsafe(value) }
    }

    pub fn is_full(&self) -> bool {
        self.buffer.is_full()
    }
}

pub struct Reader<'a, T, const N: usize> {
    buffer: &'a SpScRingbuffer<T, { N }>,
}

impl<'a, T, const N: usize> Reader<'a, T, { N }> {
    fn new(buffer: &'a SpScRingbuffer<T, { N }>) -> Self {
        Self { buffer }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn read_value(&mut self) -> Option<T> {
        unsafe { self.buffer.read_value_unsafe() }
    }

    pub fn peek_value(&mut self) -> Option<&T> {
        unsafe { self.buffer.peek_value_unsafe() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_checks() {
        let mut z = SpScRingbuffer::<u8, 4>::new();
        assert_eq!(z.is_empty(), true);
        assert_eq!(z.is_full(), false);

        assert_eq!(z.write_value(1).is_ok(), true);

        // One value in there now.
        assert_eq!(z.is_empty(), false);
        assert_eq!(z.is_full(), false);

        // read the one value
        let v = z.read_value();
        assert_eq!(v.is_some(), true);
        assert_eq!(v.unwrap(), 1);
        assert_eq!(z.is_empty(), true);
        assert_eq!(z.is_full(), false);

        assert_eq!(z.write_value(2).is_ok(), true);
        assert_eq!(z.write_value(3).is_ok(), true);
        assert_eq!(z.write_value(4).is_ok(), true);
        assert_eq!(z.write_value(5).is_ok(), false); // would make read==write

        assert_eq!(z.is_full(), true);
        assert_eq!(unsafe { z.peek_value_unsafe().is_some() }, true);
        assert_eq!(unsafe { *(z.peek_value_unsafe().unwrap()) }, 2);
        assert_eq!(z.read_value().expect("2"), 2);
        assert_eq!(z.read_value().expect("3"), 3);
        assert_eq!(z.read_value().expect("4"), 4);

        assert_eq!(z.is_empty(), true);
        assert_eq!(z.is_full(), false);

        let (mut reader, mut writer) = z.split();
        assert_eq!(reader.read_value().is_none(), true);
        assert_eq!(writer.write_value(1).is_ok(), true);
        assert_eq!(reader.read_value().expect("1"), 1);

        assert_eq!(writer.write_value(1).is_ok(), true);
        assert_eq!(writer.write_value(2).is_ok(), true);
        assert_eq!(writer.write_value(3).is_ok(), true);
        assert_eq!(writer.write_value(4).is_err(), true);
        assert_eq!(reader.read_value().expect("1"), 1);
        assert_eq!(reader.read_value().expect("2"), 2);
        assert_eq!(reader.read_value().expect("3"), 3);
        assert_eq!(reader.read_value().is_none(), true);
    }
}
