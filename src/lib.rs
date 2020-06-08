use std::boxed::Box;
use std::rc::Rc;

pub mod graphics;
use graphics::{Graphic, GraphicContext};
mod pdf;
use pdf::{Dict, Name, ObjRef, Object, PDFData};

pub struct PDF {
    pages: Vec<Page>,
    writer: pdf::PDFWrite,
    catalog: Rc<ObjRef<Dict>>,
    outlines: Rc<ObjRef<Dict>>,
    pages_obj: Rc<ObjRef<Dict>>,
}

impl PDF {
    /// Creates a new PDF file with the given output writer
    pub fn new(out: Box<dyn std::io::Write>) -> Self {
        let mut writer = pdf::PDFWrite::new(out);
        let outlines = ObjRef::new(
            0,
            Dict::from_vec(vec![("Type", Name::new("Outlines")), ("Count", Rc::new(0))]),
        );
        writer.add_object(outlines.clone());
        let pages_obj = ObjRef::new(0, Dict::from_vec(vec![("Type", Name::new("Pages"))]));
        writer.add_object(pages_obj.clone());
        Self {
            pages: vec![],
            catalog: writer.create_root(Dict::from_vec(vec![
                ("Type", Name::new("Catalog")),
                ("Outlines", outlines.clone()),
                ("Pages", pages_obj.clone()),
            ])),
            outlines,
            pages_obj,
            writer,
        }
    }
    /// Creates a new PDF file, using the file as a writer to write to
    pub fn from_file(file: std::fs::File) -> Self {
        Self::new(Box::new(file))
    }
    /// Adds a page to the PDF
    ///
    /// The page is consumed, and may (or may not)
    /// be written to the output right away.
    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
    }
    /// Completes the writing process
    ///
    /// TODO: this may be added to a drop implementation
    pub fn write(mut self) -> std::io::Result<()> {
        let (pg_obj, tmp) = (&mut self.pages_obj, &mut self.writer);
        let p: Vec<Rc<dyn Object>> = self
            .pages
            .into_iter()
            .map(|p| {
                let page = p.render(pg_obj.clone(), tmp);
                tmp.add_object(page)
            })
            .collect();
        self.pages_obj.add_entry("Count", Rc::new(p.len()));
        self.pages_obj.add_entry("Kids", Rc::new(p));

        self.writer.write()
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
    pub fn add(&mut self, g: Rc<impl Graphic>) {
        self.graphics.render(g);
    }
    fn render(self, parent: Rc<dyn PDFData>, write: &mut pdf::PDFWrite) -> Rc<dyn Object> {
        let (streams, resources) = self.graphics.compile(write);
        if streams.len() == 1 {
            ObjRef::new(
                0,
                Dict::from_vec(vec![
                    ("Type", Name::new("Page")),
                    ("Parent", parent),
                    (
                        "MediaBox",
                        graphics::Rect::new(0f64, 0f64, 612f64, 792f64).as_data(),
                    ),
                    ("Contents", streams[0].clone()),
                    ("Resources", resources),
                ]),
            )
        } else {
            ObjRef::new(
                0,
                Dict::from_vec(vec![
                    ("Type", Name::new("Page")),
                    ("Parent", parent),
                    (
                        "MediaBox",
                        graphics::Rect::new(0f64, 0f64, 612f64, 792f64).as_data(),
                    ),
                    ("Contents", Rc::new(streams.clone())),
                    ("Resources", resources),
                ]),
            )
        }
    }
}
