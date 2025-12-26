mod backend;
mod indexing;

use backend::cache::Cache;
use backend::pager::Pager;
use indexing::table_heap::TableHeap;
use indexing::table_iterator::TableIterator;
use std::fs;
use std::sync::Arc;

fn main() {
    let file_name = "mydb.db";
    if std::path::Path::new(file_name).exists() {
        fs::remove_file(file_name).unwrap();
    }

    println!("--- 1. Initializing Database ---");
    let pager = Pager::new(file_name).expect("Failed to create pager");
    let cache = Arc::new(Cache::new(pager, 3));

    {
        let frame = cache.fetch_page(0).expect("Should create Page 0");
        let mut frame_guard = frame.write().unwrap();
        // Initialize it as a TablePage (headers, etc)
        if let Some(ref mut page) = frame_guard.page {
            let mut tp = indexing::table_page::TablePage::new(&mut page.data);
            tp.init(0, u32::MAX); // Page 0, No Prev Page
        }
    }
    cache.unpin_page(0, true);
    let table_heap = Arc::new(TableHeap::new(cache.clone(), 0));

    println!("--- 2. Inserting Data ---");
    // Insert 1000 tuples.
    let count = 1000;
    for i in 0..count {
        let msg = format!("Tuple #{}", i);
        let tuple_data = msg.as_bytes();

        // Insert returns (PageID, SlotID)
        match table_heap.insert(tuple_data) {
            Ok((pid, sid)) => {
                if i % 200 == 0 {
                    println!("Inserted {} at Page {}, Slot {}", msg, pid, sid);
                }
            }
            Err(e) => panic!("Insertion failed: {}", e),
        }
    }

    println!("--- 3. Scanning Data (Iterator) ---");
    // Create the iterator
    let mut iterator = TableIterator::new(table_heap.clone(), 0);

    let mut read_count = 0;
    while let Some(tuple_bytes) = iterator.next() {
        let msg = String::from_utf8(tuple_bytes).unwrap();

        if !msg.starts_with("Tuple #") {
            panic!("Read corrupted data: {}", msg);
        }

        read_count += 1;
    }

    println!("Total Tuples Read: {}", read_count);

    if read_count == count {
        println!("✅ SUCCESS: Read back all {} tuples!", count);
    } else {
        println!("❌ FAILURE: Expected {}, but read {}", count, read_count);
    }
}
