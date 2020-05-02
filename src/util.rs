pub mod fs {
    use std::fs::File;
    use std::io;
    use std::io::Read;
    
    pub fn file_buffer_size(file: &File) -> usize {
        // Allocate one extra byte so the buffer doesn't need to grow before the
        // final `read` call at the end of the file.  Don't worry about `usize`
        // overflow because reading will fail regardless in that case.
        file.metadata()
            .map(|it| it.len() as usize + 1)
            .unwrap_or(0)
    }
    
    pub fn read_all(file: &mut File) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(file_buffer_size(&file));
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

pub mod future {
    use futures::future::JoinAll;
    use futures::Future;
    
    pub trait FutureIterator: Iterator {
        // can't use async fn's in traits b/c of impl Future,
        // but I can when the return Future type is concrete
        fn join_all(self) -> JoinAll<<Self as IntoIterator>::Item>
            where
                Self: Sized,
                Self: IntoIterator,
                <Self as IntoIterator>::Item: Future {
            futures::future::join_all(self)
        }
    }
    
    impl<T: ?Sized + Iterator> FutureIterator for T {}
}
