mod backend;
mod indexing;

use crate::backend::pager::Pager;

fn main() -> std::io::Result<()> {
    let mut pager = Pager::new("mydb.db")?;

    let page = pager.read_page(0)?;
    println!("Successfully read Page #{}", page.id);
    println!("First byte: {}", page.data[0]);

    Ok(())
}
