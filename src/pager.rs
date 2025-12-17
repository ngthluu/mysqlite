use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom};

pub const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;

type Page = [u8; PAGE_SIZE];

pub struct Pager {
    file: File,
    file_length: u64,
    pages: Vec<Option<Page>>,
    num_pages: u32,
}

impl Pager {
    pub fn new(filename: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)?;

        let file_length = file.metadata()?.len();
        let num_pages = (file_length / PAGE_SIZE as u64) as u32;

        let mut pages = Vec::with_capacity(TABLE_MAX_PAGES);
        pages.resize_with(TABLE_MAX_PAGES, || None);

        Ok(Self {
            file,
            file_length,
            pages,
            num_pages,
        })
    }

    pub fn get_page(&mut self, page_num: u32) -> io::Result<&mut Page> {
        if page_num as usize >= TABLE_MAX_PAGES {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "Exceeded table max pages",
            ));
        }

        // Check cache
        if self.pages[page_num as usize].is_none() {
            // Cache missed
            let mut page = [0; PAGE_SIZE];
            let page_offset = page_num as u64 * PAGE_SIZE as u64;

            if page_num < self.num_pages {
                self.file.seek(SeekFrom::Start(page_offset))?;
                self.file.read_exact(&mut page)?;
            } else if page_num == self.num_pages {
                self.num_pages += 1;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Page {} not found.", page_num),
                ));
            }
            self.pages[page_num as usize] = Some(page);
        }

        match self.pages[page_num as usize].as_mut() {
            Some(page) => Ok(page),
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "Page cache logic failed",
            )),
        }
    }
}
