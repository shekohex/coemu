pub fn current_ts() -> u32 {
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() as u32
}

pub trait LoHi {
    type Output;

    fn lo(&self) -> Self::Output;
    fn hi(&self) -> Self::Output;
    fn constract(hi: Self::Output, lo: Self::Output) -> Self;
}

impl LoHi for u16 {
    type Output = u8;

    fn lo(&self) -> Self::Output {
        *self as u8
    }

    fn hi(&self) -> Self::Output {
        (*self >> 8) as u8
    }

    fn constract(hi: Self::Output, lo: Self::Output) -> Self {
        lo as u16 | (hi as u16) << 8
    }
}

impl LoHi for u32 {
    type Output = u16;

    fn lo(&self) -> Self::Output {
        *self as u16
    }

    fn hi(&self) -> Self::Output {
        (*self >> 16) as u16
    }

    fn constract(hi: Self::Output, lo: Self::Output) -> Self {
        lo as u32 | (hi as u32) << 16
    }
}

impl LoHi for u64 {
    type Output = u32;

    fn lo(&self) -> Self::Output {
        *self as u32
    }

    fn hi(&self) -> Self::Output {
        (*self >> 32) as u32
    }

    fn constract(hi: Self::Output, lo: Self::Output) -> Self {
        lo as u64 | (hi as u64) << 32
    }
}
