use core::marker::Copy;

pub struct WriteOverrun();
pub struct ReadOverrun();

/*
    if read_pos == write_pos, no data.
    write_pos denotes where where we are going to write.
    read_pos denotes up to where we have read.
*/

/// Simple ringbuffer, holds up to N - 1 elements.
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

    #[cfg(test)]
    pub fn set_read_pos(&mut self, v: usize) {
        self.read_pos = v;
    }

    #[cfg(test)]
    pub fn set_write_pos(&mut self, v: usize) {
        self.write_pos = v;
    }

    #[cfg(test)]
    pub fn array(&mut self) -> &mut [T] {
        &mut self.array[..]
    }

    pub fn read_pos(&self) -> usize {
        self.read_pos
    }
    pub fn write_pos(&self) -> usize {
        self.write_pos
    }

    /// Get the number of entries that are at maximum available for writing.
    pub fn write_available(&self) -> usize {
        if self.write_pos < self.read_pos {
            self.read_pos - self.write_pos - 1
        } else {
            (self.write_pos..(if self.read_pos == 0 { N - 1 } else { N })).len()
                + self.read_pos.saturating_sub(1)
        }
    }

    /// Get longest available writable slice without destroying data not read yet.
    pub fn write_slice_mut<'a>(&'a mut self) -> &'a mut [T] {
        // Easy case;
        if self.write_pos < self.read_pos {
            // writeable is between write_pos and read_pos - 1.
            &mut self.array[self.write_pos..(self.read_pos - 1)]
        } else {
            // Else, it is always between current write pos and N - 1, OR N if read_pos is not 0.
            &mut self.array[self.write_pos..(if self.read_pos == 0 { N - 1 } else { N })]
        }
    }

    /// Advance write index by certain value.
    pub fn write_advance(&mut self, count: usize) -> Result<(), WriteOverrun> {
        let available = self.write_available();
        if count > available {
            return Err(WriteOverrun());
        }
        self.write_pos = (self.write_pos + count) % N;
        Ok(())
    }

    /// Write a value to the ringbuffer, advancing the write position.
    pub fn write_value(&mut self, value: T) -> Result<(), WriteOverrun> {
        let available = self.write_available();
        if available == 0 {
            return Err(WriteOverrun());
        }
        self.array[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % N;
        Ok(())
    }

    /// Get the number of entries that are at least available for read.
    pub fn read_available(&self) -> usize {
        if self.write_pos < self.read_pos {
            // Readable buffer goes across wrap.
            (N - self.read_pos) + self.write_pos
        } else {
            // Readable buffer is difference between write and read.
            self.write_pos - self.read_pos
        }
    }

    /// Read a value out of the ringbuffer, advancing the read position.
    pub fn read_value(&mut self) -> Option<T> {
        if self.read_available() != 0 {
            let v = self.array[self.read_pos];
            self.read_pos = (self.read_pos + 1) % N;
            return Some(v);
        }
        None
    }
    /// Get the longest available read slice.
    pub fn read_slice_mut<'a>(&'a mut self) -> &'a mut [T] {
        if self.write_pos < self.read_pos {
            &mut self.array[self.read_pos..N]
        } else {
            // Values between read and write.
            &mut self.array[self.read_pos..self.write_pos]
        }
    }

    /// Advance write index by certain value.
    pub fn read_advance(&mut self, count: usize) -> Result<(), ReadOverrun> {
        let available = self.read_available();
        if count > available {
            return Err(ReadOverrun());
        }
        self.read_pos = (self.read_pos + count) % N;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stateless_checks() {
        // 0 1 2 3
        let mut z = RingBuffer::<u8, 4>::new();

        // Indices the same == no data.
        z.set_read_pos(0);
        z.set_write_pos(0);

        // 0 1 2 3
        //R
        // W
        assert_eq!(z.read_available(), 0);
        assert_eq!(z.read_slice_mut().len(), 0);
        assert_eq!(z.write_available(), 3);
        assert_eq!(z.write_slice_mut().len(), 3);

        // Trivial case, read at 0, write at 2
        // Slots available for reading: 0 and 1.
        // Slots available for writing; 2 (not 3, because then W would advance to R)
        // 0 1 2 3
        //R
        //     W
        z.set_read_pos(0);
        z.set_write_pos(2);
        assert_eq!(z.read_available(), 2);
        assert_eq!(z.read_slice_mut().len(), 2);
        assert_eq!(z.write_available(), 1);
        assert_eq!(z.write_slice_mut().len(), 1);

        // Non trivial case, read at 2, write at 0
        // Slots available for reading: 2, 3
        // Slots available for writing: 0, (not 1 because then W would advance to R)
        // 0 1 2 3
        //    R
        // W
        z.set_read_pos(2);
        z.set_write_pos(0);
        assert_eq!(z.read_available(), 2);
        assert_eq!(z.read_slice_mut().len(), 2);
        assert_eq!(z.write_available(), 1);
        assert_eq!(z.write_slice_mut().len(), 1);

        // Index: 0 1 2 3
        //       R
        //              W
        // All writes populated.
        // We cannot write 3, because that makes read == write, meaning empty.
        // 0, 1, 2 ready for reading.
        // no slots for writing.
        z.set_read_pos(0);
        z.set_write_pos(3);
        assert_eq!(z.read_available(), 3);
        assert_eq!(z.read_slice_mut().len(), 3);
        assert_eq!(z.write_available(), 0);
        assert_eq!(z.write_slice_mut().len(), 0);

        // Index: 0 1 2 3
        //         R
        //              W
        // All writes populated.
        // 1, 2 for reading
        // 0 for writing.
        z.set_read_pos(1);
        z.set_write_pos(3);
        assert_eq!(z.read_available(), 2);
        assert_eq!(z.read_slice_mut().len(), 2);
        assert_eq!(z.write_available(), 1);

        z.set_read_pos(2);
        z.set_write_pos(2);
        // Index: 0 1 2 3
        //           R
        //            W
        // All writes populated.
        // 3, 0, 1 available for writing.
        assert_eq!(z.read_available(), 0);
        assert_eq!(z.read_slice_mut().len(), 0);
        assert_eq!(z.write_available(), 3);
        assert_eq!(z.write_slice_mut().len(), 2);
    }

    #[test]
    fn state_checks() {
        // Index: 0 1 2 3
        //       R
        //        W
        // Val:   0 0 0 0
        let mut z = RingBuffer::<u8, 4>::new();
        assert_eq!(z.read_available(), 0);
        assert_eq!(z.write_available(), 3);
        assert_eq!(z.array(), &[0, 0, 0, 0]);

        // Write things.
        let v = z.write_slice_mut();
        assert_eq!(v.len(), 3);
        v[0] = 1;
        v[1] = 2;
        assert_eq!(z.write_advance(2).is_ok(), true);

        // Index: 0 1 2 3
        //       R
        //            W
        // Val:   1 2 0 0
        assert_eq!(z.read_available(), 2);
        assert_eq!(z.write_available(), 1);
        assert_eq!(z.array(), &[1, 2, 0, 0]);
        assert_eq!(z.read_slice_mut(), &[1, 2]);

        // Write things.
        let v = z.write_slice_mut();
        assert_eq!(v.len(), 1);
        v[0] = 3;
        assert_eq!(z.write_advance(2).is_err(), true);
        assert_eq!(z.write_advance(1).is_ok(), true);

        // Index: 0 1 2 3
        //       R
        //              W
        // Val:   1 2 3 0
        assert_eq!(z.read_available(), 3);
        assert_eq!(z.write_available(), 0);
        assert_eq!(z.array(), &[1, 2, 3, 0]);
        assert_eq!(z.write_value(10).is_err(), true);

        // Read a value.
        let r = z.read_value();
        assert_eq!(r.is_some(), true);
        assert_eq!(r.unwrap(), 1);
        // Index: 0 1 2 3
        //         R
        //              W
        // Val:   1 2 3 0
        assert_eq!(z.read_available(), 2);
        assert_eq!(z.write_available(), 1);
        assert_eq!(z.array(), &[1, 2, 3, 0]);
        assert_eq!(z.read_slice_mut(), &[2, 3]);

        let r = z.read_value();
        assert_eq!(r.is_some(), true);
        assert_eq!(r.unwrap(), 2);
        // Index: 0 1 2 3
        //           R
        //              W
        // Val:   1 2 3 0
        assert_eq!(z.read_available(), 1);
        assert_eq!(z.write_available(), 2);
        assert_eq!(z.array(), &[1, 2, 3, 0]);
        assert_eq!(z.read_slice_mut(), &[3]);

        // Add a value.
        let r = z.write_value(4);
        assert_eq!(r.is_ok(), true);
        // let v = z.write_slice_mut();
        // assert_eq!(v.len(), 1);
        // v[0] = 4;
        // assert_eq!(z.write_advance(1).is_ok(), true);
        // Index: 0 1 2 3
        //           R
        //        W
        // Val:   1 2 3 4
        assert_eq!(z.read_available(), 2);
        assert_eq!(z.write_available(), 1);
        assert_eq!(z.array(), &[1, 2, 3, 4]);

        assert_eq!(z.read_slice_mut(), &[3, 4]);
        assert_eq!(z.read_advance(2).is_ok(), true);
    }
}
