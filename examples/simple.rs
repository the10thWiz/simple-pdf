use simple_pdf::{graphics, Page, PDF};
use std::fs::File;

fn main() -> std::io::Result<()> {
    let mut pdf = PDF::new();
    let mut page = Page::new(); // Page builder
    page.add(
        graphics::Path::from((10f64, 10f64))
            .line_to((200f64, 200f64))
            .rect((10f64, 10f64, 190f64, 190f64))
            .stroke(graphics::Color::red()),
    );
    pdf.add_page(page);
    let mut output = File::create("simple")?;
    pdf.write(&mut output)
}
