use core::marker::Copy;

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

impl<T, const N: usize> SpScRingbuffer<T,  N > {
    const VAL: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

    pub fn new() -> Self {
        // Odd workaround for E0277
        SpScRingbuffer::<T,  N > {
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

    // #[cfg(test)]
    // pub fn array(&mut self) -> &mut [T] {
        // &mut self.array[..]
    // }


    pub fn is_empty(&self) -> bool {
        self.read_pos.load(Ordering::Relaxed) == self.write_pos.load(Ordering::Relaxed)
    }

    pub fn is_full(&self) -> bool {
        ((self.write_pos.load(Ordering::Relaxed) + 1) % N) == self.read_pos.load(Ordering::Relaxed)
    }

    /// Read a value out of the SpScQueue, advancing the read position.
    pub fn read_value(&mut self) -> Option<T> {

        // check if there's something to return
        if self.is_empty() {
            return None;
        }
        // Get the read position.
        let r_pos = self.read_pos.load(Ordering::Relaxed);

        // Rip the value out of the array
        let v = unsafe { self.array[r_pos].get().read().assume_init() };

        // Update the new read position.
        self.read_pos.store((r_pos + 1) % N, Ordering::Release);
        Some(v)
    }

    pub fn write_value(&mut self, value: T) -> Result<(), WriteOverrun>
    {
        // Check if we can write
        if self.is_full() {
            return Err(WriteOverrun());
        }

        // Obtain the write position.
        let w_pos = self.write_pos.load(Ordering::Relaxed);

        // Insert the value into the array.
        (self.array[w_pos].get_mut()).write(value);

        // Advance the write pointer.
        self.write_pos.store((w_pos + 1) % N, Ordering::Release);
        Ok(())
    }


    /*
    pub fn split<'a>(&'a mut self) -> (Reader<'a, T, N>, Writer<'a, T, N>) {
        (Reader::new(self), Writer::new(self))
    }
    */
}

/*
pub struct Writer<'a, T: Copy + Default, const N: usize> {
    buffer: &'a SpScQueue<T, { N }>,
}

impl<'a, T: Copy + Default, const N: usize> Writer<'a, T, { N }> {
    fn new(buffer: &'a SpScQueue<T, { N }>) -> Self {
        Self{buffer}
    }

    pub fn write_value(&mut self, value: T) -> Result<(), WriteOverrun> {
        unsafe {
            self.buffer.write_value_unsafe(value)
        }
    }
}

pub struct Reader<'a, T: Copy + Default, const N: usize> {
    buffer: &'a SpScQueue<T, { N }>,
}

impl<'a, T: Copy + Default, const N: usize> Reader<'a, T, { N }> {
    fn new(buffer: &'a SpScQueue<T, { N }>) -> Self {
        Self{buffer}
    }

    pub fn read_value(&mut self) -> Option<T> {
        unsafe {
            self.buffer.read_value_unsafe()
        }
    }
}
*/

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
        assert_eq!(z.read_value().expect("2"), 2);
        assert_eq!(z.read_value().expect("3"), 3);
        assert_eq!(z.read_value().expect("4"), 4);

        assert_eq!(z.is_empty(), true);
        assert_eq!(z.is_full(), false);
    }
}



