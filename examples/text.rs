use simple_pdf::{graphics, Page, PDF};
use std::fs::File;

fn main() -> std::io::Result<()> {
    let mut pdf = PDF::from_file(File::create("text")?);
    let mut page = Page::new();
    // page.add(
    //     graphics::Text::new(graphics::Font::times_new_roman(), 12f64)
    //         .move_to((100f64, 100f64))
    //         .fill(graphics::Color::red())
    //         .text("Hello World!"),
    // );
    pdf.add_page(page);
    pdf.write()
}
