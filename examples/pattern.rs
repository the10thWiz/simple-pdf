use simple_pdf::{graphics, Page, PDF};
use std::fs::File;

fn main() -> std::io::Result<()> {
    let mut pdf = PDF::from_file(File::create("pattern")?);
    let mut page = Page::new(); // Page builder
    page.add(
        graphics::Path::new()
            .rect((10f64, 10f64, 190f64, 190f64))
            .fill(graphics::Color::red()),
    );
    pdf.add_page(page);
    pdf.write()
}
