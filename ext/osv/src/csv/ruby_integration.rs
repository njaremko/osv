use std::{fs::File, io, mem::ManuallyDrop};

pub struct ForgottenFileHandle(pub ManuallyDrop<File>);

impl std::io::Read for ForgottenFileHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    // fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
    //     self.0.read_buf(cursor)
    // }

    // #[inline]
    // fn is_read_vectored(&self) -> bool {
    //     self.0.is_read_vectored()
    // }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.0.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.0.read_to_string(buf)
    }
}
