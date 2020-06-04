use std::rc::Rc;

pub mod graphics;
use graphics::{Graphic, GraphicContext};
pub mod pdf;
use pdf::{Dict, Name, Object};

pub struct PDF {
    pages: Vec<Page>,
}

impl PDF {
    pub fn new() -> Self {
        Self { pages: vec![] }
    }
    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
    }
    pub fn write(self, out: &mut dyn std::io::Write) -> std::io::Result<()> {
        let mut pdf_writer = pdf::PDFWrite::new();
        /* << /Type /Outlines
            /Count 0
        >> */
        let outlines = Object::new(
            0,
            Dict::from_vec(vec![("Type", Name::new("Outlines")), ("Count", Rc::new(0))]),
        );
        let pages = Object::empty(0);
        let p: Vec<Rc<Object>> = self
            .pages
            .into_iter()
            .map(|p| {
                let (page, contents) = p.render(pages.clone());
                for obj in contents {
                    pdf_writer.add_object(obj);
                }
                pdf_writer.add_object(page)})
            .collect();
        pages.assign(Dict::from_vec(vec![
            ("Type", Name::new("Pages")),
            ("Count", Rc::new(p.len())),
            ("Kids", Rc::new(p)),
        ]));
        let catalog = Object::new(
            0,
            Dict::from_vec(vec![
                ("Type", Name::new("Catalog")),
                ("Outlines", outlines.clone()),
                ("Pages", pages.clone()),
            ]),
        );
        pdf_writer.add_object(outlines);
        pdf_writer.set_root(catalog);
        pdf_writer.add_object(pages);
        pdf_writer.write(out)
    }
}

pub struct Page {
    // elements: Vec<Box<dyn Graphic>>,
    graphics: GraphicContext,
}

impl Page {
    pub fn new() -> Self {
        Self {
            // elements: vec![],
            graphics: GraphicContext::new(),
        }
    }
    pub fn add(&mut self, g: &impl Graphic) {
        self.graphics.render(g);
    }
    fn render(self, parent: Rc<Object>) -> (Rc<Object>, Vec<Rc<Object>>) {
        let streams = self.graphics.compile();
        if streams.len() == 1 {
            (Object::new(
                0,
                Dict::from_vec(vec![
                    ("Type", Name::new("Page")),
                    ("Parent", parent),
                    (
                        "MediaBox",
                        graphics::Rect::new(0f64, 0f64, 612f64, 792f64).as_data(),
                    ),
                    ("Contents", streams[0].clone()),
                    (
                        "Resources",
                        Dict::from_vec(vec![(
                            "ProcSet",
                            Rc::new(vec![Name::new("PDF"), Name::new("Text")]),
                        )]),
                    ),
                ]),
            ), streams)
        } else {
            (Object::new(
                0,
                Dict::from_vec(vec![
                    ("Type", Name::new("Page")),
                    ("Parent", parent),
                    (
                        "MediaBox",
                        graphics::Rect::new(0f64, 0f64, 612f64, 792f64).as_data(),
                    ),
                    ("Contents", Rc::new(streams.clone())),
                    (
                        "Resources",
                        Dict::from_vec(vec![(
                            "ProcSet",
                            Rc::new(vec![Name::new("PDF"), Name::new("Text")]),
                        )]),
                    ),
                ]),
            ), streams)
        }
        
        // /Type /Page
        // /Parent 3 0 R
        // /MediaBox [0 0 612 792]
        // /Contents 5 0 R
        // /Resources << /ProcSet 6 0 R
        //                 /Font << /F1 7 0 R>>
        //             >>
    }
}
