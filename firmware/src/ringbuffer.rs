

use core::marker::Copy;

pub enum RingBufferState {
    WriteOverrun,
}

/// Simple ringbuffer.
pub struct RingBuffer<T: Copy + Default, const N: usize> {
    array: [T; N],
    read_pos: usize,
    write_pos: usize,
}

impl<T: Copy + Default, const N: usize> RingBuffer<T, { N }> {
    pub fn new() -> Self {
        RingBuffer::<T, { N }> {
            array: [Default::default(); N],
            read_pos: 0,
            write_pos: 0,
        }
    }

    /// Get the number of entries that are at least available for read.
    pub fn read_available(&self) -> usize {
        if self.write_pos < self.read_pos {
            // Readable buffer goes across wrap.
            // Difference is wrap - read, plus write pos after wrap.
            (N - self.read_pos) + self.write_pos
        } else {
            // Readable buffer is difference between write and read.
            self.write_pos - self.read_pos
        }
    }

    /// Get longest available writable slice without destroying data not read yet.
    pub fn write_slice_mut<'a>(&'a mut self) -> &'a mut [T] {
        if self.write_pos < self.read_pos {
            // writeable is between write_pos and read_pos.
            &mut self.array[self.write_pos..self.read_pos]
        } else {
            // Writable is between write_pos and wrap
            &mut self.array[self.write_pos..N]
        }
    }

    /// Advance write index by certain value.
    pub fn write_advance(&mut self, count: usize) -> Result<(), RingBufferState> {
        if self.write_pos < self.read_pos {
            if (self.write_pos + count) > self.read_pos {
                return Err(RingBufferState::WriteOverrun);
            }
        } else {
            // read_pos <= write_pos
            let allowable = (N - self.write_pos) + self.read_pos;
            if ((self.write_pos + count) % N) > allowable {
                return Err(RingBufferState::WriteOverrun);
            }
        }
        self.write_pos = (self.write_pos + count) % N;
        Ok(())
    }

    /// Read a value out of the ringbuffer, advancing the read pointer.
    pub fn read_value(&mut self) -> Option<T> {
        if self.read_available() != 0 {
            let v = self.array[self.read_pos];
            self.read_pos = (self.read_pos + 1) % N;
            return Some(v);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foo() {

        let mut z = RingBuffer::<u8, 6>::new();
        assert_eq!(z.read_available(), 0);
        assert_eq!(z.write_slice_mut().len(), 6);
        assert_eq!(z.read_value().is_none(), true);

        let mut w = z.write_slice_mut();

        w[0] = 0;
        w[1] = 1;
        assert_eq!(z.write_advance(2).is_ok(), true);
        assert_eq!(z.read_available(), 2);
        assert_eq!(z.write_slice_mut().len(), 4);

        let v0 = z.read_value();
        assert_eq!(v0.is_some(), true);
        assert_eq!(v0.unwrap(), 0);
        assert_eq!(z.read_available(), 1);
        // slice doesn't change, consecutive is only to the wrap.
        assert_eq!(z.write_slice_mut().len(), 4);
        let mut w = z.write_slice_mut();
        w[0] = 2;
        w[1] = 3;
        assert_eq!(z.write_advance(2).is_ok(), true);
        assert_eq!(z.read_available(), 3);
        assert_eq!(z.write_slice_mut().len(), 2);

        let v0 = z.read_value();
        assert_eq!(v0.is_some(), true);
        assert_eq!(v0.unwrap(), 1);

        let mut w = z.write_slice_mut();
        assert_eq!(w.len(), 2);
        w[0] = 2;
        w[1] = 3;
        assert_eq!(z.write_advance(2).is_ok(), true); // written up to wrap

        // Buffer is now beyond the wraparound
        let mut w = z.write_slice_mut();
        assert_eq!(w.len(), 2);

        // if we read one, the buffer should increase now.
        let v0 = z.read_value();
        assert_eq!(v0.is_some(), true);
        assert_eq!(v0.unwrap(), 2);

        let mut w = z.write_slice_mut();
        assert_eq!(w.len(), 3);
        w[0] = 4;
        w[1] = 5;
        w[2] = 6;

        assert_eq!(z.write_advance(3).is_ok(), true); // completely full.

         println!("testprint");
        let mut w = z.write_slice_mut();
        assert_eq!(w.len(), 3);
        assert_eq!(z.write_advance(1).is_err(), true); // completely full.
        


    }
}
