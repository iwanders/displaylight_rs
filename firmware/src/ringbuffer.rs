
pub struct RingBuffer<T, const N: usize> {
    array: core::mem::MaybeUninit<[T; N]>,
    read_pos: usize,
    write_pos: usize,
}

impl<T, const N: usize>  RingBuffer<T, { N }> {
    pub fn new() -> Self {
        RingBuffer::<T, { N }>{
            array: unsafe{core::mem::uninitialized()},
            read_pos: 0,
            write_pos: 0,
        }
    }
}


#[cfg(test)]
mod tests{

#[test]
fn foo()
{
}
}