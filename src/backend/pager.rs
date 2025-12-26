use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};

// Page size is 4096 bytes (4KB)
pub const PAGE_SIZE: usize = 4096;

// Page data includes PAGE_SIZE bytes (u8 = 1 byte)
pub type PageData = [u8; PAGE_SIZE];

#[derive(Debug)]
pub struct Page {
    pub id: usize,
    pub data: PageData,
}

// Pager
// Responsible for persist / reading data from disk
// Reading data via Page
#[derive(Debug)]
pub struct Pager {
    file: File,
}

impl Pager {
    pub fn new(filename: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)?;

        Ok(Self { file })
    }

    // Get the total number of pages
    // currently in the file
    pub fn page_count(&self) -> io::Result<usize> {
        let metadata = self.file.metadata()?;
        let len = metadata.len();
        let page_count = (len as usize) / PAGE_SIZE;
        Ok(page_count)
    }

    // Read page with page_id (from 0)
    // Calculate the offset: offset = page_id * PAGE_SIZE
    // Seek to the offset and read the whole page
    pub fn read_page(&mut self, page_id: usize) -> io::Result<Page> {
        // Check: page_id bigger than page_count
        // Raise exception
        let page_count = self.page_count()?;
        if page_id >= page_count {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Page {} does not exist. File has {} pages.",
                    page_id, page_count
                ),
            ));
        }

        let offset = (page_id * PAGE_SIZE) as u64;

        self.file.seek(SeekFrom::Start(offset))?;
        let mut data = [0u8; PAGE_SIZE];
        self.file.read_exact(&mut data)?;

        Ok(Page { id: page_id, data })
    }

    // Write page
    // Reverse to the `read_page`
    pub fn write_page(&mut self, page: &Page) -> io::Result<()> {
        let offset = (page.id * PAGE_SIZE) as u64;

        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&page.data)?;

        Ok(())
    }
}
