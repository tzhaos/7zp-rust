use std::{
    io::{self, Read},
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

#[derive(Clone)]
pub struct Progress(Arc<AtomicU8>);

impl Progress {
    pub fn new() -> Self {
        Self(Arc::new(AtomicU8::new(u8::MAX)))
    }

    pub fn percent(&self) -> Option<u8> {
        match self.0.load(Ordering::Relaxed) {
            u8::MAX => None,
            value => Some(value),
        }
    }
}

pub(super) fn read(mut pipe: impl Read, progress: &Progress) -> io::Result<Vec<u8>> {
    let mut buffer = [0; 4096];
    let mut number = 0u32;
    let mut digits = 0;
    let mut prefix = true;
    // 7-Zip 26.03 on Windows rewrites each progress record with carriage returns.
    loop {
        let count = pipe.read(&mut buffer)?;
        if count == 0 {
            return Ok(Vec::new());
        }
        for &byte in &buffer[..count] {
            if matches!(byte, b'\r' | b'\n') {
                number = 0;
                digits = 0;
                prefix = true;
            } else if prefix && byte.is_ascii_digit() {
                if digits < 3 {
                    number = number * 10 + u32::from(byte - b'0');
                }
                digits = (digits + 1).min(4);
            } else if prefix && !(byte == b' ' && digits == 0) {
                if byte == b'%' && (1..=3).contains(&digits) && number <= 100 {
                    progress.0.store(number as u8, Ordering::Relaxed);
                }
                prefix = false;
            }
        }
    }
}
