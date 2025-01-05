use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    mem::ManuallyDrop,
};

pub struct ForgottenFileHandle(pub ManuallyDrop<File>);

impl Read for ForgottenFileHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Seek for ForgottenFileHandle {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}
