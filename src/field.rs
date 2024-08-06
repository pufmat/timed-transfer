pub struct Field<T> {
    offset: usize,
    mask: T,
}

pub const fn bit(bit: usize) -> Field<u32> {
    bits(bit, bit)
}

pub const fn bits(end: usize, start: usize) -> Field<u32> {
    Field {
        offset: start,
        mask: (((1 << (end - start + 1)) - 1) << start) as u32,
    }
}

#[inline(always)]
pub fn read_bit_field(src: u32, field: Field<u32>) -> u32 {
    (src & field.mask) >> field.offset
}

#[inline(always)]
pub fn write_bit_field(dst: &mut u32, field: Field<u32>, value: impl Into<u32>) {
    *dst = (*dst & !field.mask) | (value.into() << field.offset);
}
