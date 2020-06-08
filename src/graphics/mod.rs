use crate::pdf::{Dict, Name, Object};
use std::collections::LinkedList;
use std::rc::Rc;
pub mod path;
pub use path::Path;
pub mod text;
pub use text::{Font, Text};
pub mod context;
use context::GraphicParameters;
pub use context::{Color, Graphic, GraphicsContextType, Point, Rect};

#[derive(Debug)]
pub struct GraphicContext {
    // Mutable state
    current: Rc<GraphicParameters>,
    stack: LinkedList<GraphicParameters>,
    // Output stream
    stream: Vec<u8>,
    // Resource Dict
    resources: Rc<Dict>,
    fonts: Rc<Dict>,
    external_resources: Vec<Rc<dyn Object>>,
}
impl GraphicContext {
    pub fn new() -> Self {
        Self {
            current: Rc::new(GraphicParameters::default()),
            stack: LinkedList::new(),
            stream: vec![],
            resources: Dict::from_vec(vec![(
                "ProcSet",
                Rc::new(vec![Name::new("PDF"), Name::new("Text")]),
            )]),
            fonts: Dict::new(),
            external_resources: vec![],
        }
    }
    fn with_type(t: GraphicsContextType) -> Self {
        Self {
            current: Rc::new(GraphicParameters::with_type(t)),
            stack: LinkedList::new(),
            stream: vec![],
            resources: Dict::from_vec(vec![(
                "ProcSet",
                Rc::new(vec![Name::new("PDF"), Name::new("Text")]),
            )]),
            fonts: Dict::new(),
            external_resources: vec![],
        }
    }
    pub fn render(&mut self, object: Rc<impl Graphic>) {
        // Check Colors, and update as needed
        GraphicParameters::update(self, object.get_graphics_parameters());
        // Render object
        object.render(self);
    }
    fn command(&mut self, params: &mut [Parameter], operator: &str) {
        for p in params {
            self.stream.push(' ' as u8);
            self.stream.append(&mut p.raw);
        }
        self.stream.push(' ' as u8);
        self.stream.extend(operator.bytes());
    }
    fn add_resource(&mut self, obj: Rc<dyn Object>) {
        self.external_resources.push(obj);
    }
    fn add_font(&mut self, f: Rc<text::Font>) {
        // self.fonts.add_entry(f.name(), f.object());
        // self.external_resources.push(f.object());
    }
    pub fn compile(
        self,
        // write: &mut crate::pdf::PDFWrite,
    ) -> (
        Vec<Rc<crate::pdf::ObjRef<crate::pdf::types::Stream>>>,
        Rc<crate::pdf::Dict>,
    ) {
        if !self.fonts.is_empty() {
            self.resources.add_entry("Font", self.fonts);
        }

        let streams = vec![crate::pdf::ObjRef::new(
            0,
            crate::pdf::types::Stream::new(crate::pdf::Dict::new(), self.stream),
        )];
        // for obj in streams.iter().cloned() {
        //     write.add_object(obj);
        // }
        // for obj in self.external_resources {
        //     write.add_object(obj);
        // }
        (streams, self.resources)
    }
}

/// A raw, compiled representation of a set of parameters
///
/// Should never have trailing whitespace
pub struct Parameter {
    raw: Vec<u8>,
}

impl From<&str> for Parameter {
    fn from(o: &str) -> Self {
        Self {
            raw: format!("({})", o).bytes().collect(),
        }
    }
}

impl From<&String> for Parameter {
    fn from(o: &String) -> Self {
        Self {
            raw: format!("({})", o).bytes().collect(),
        }
    }
}
impl From<String> for Parameter {
    fn from(o: String) -> Self {
        Self {
            raw: format!("({})", o).bytes().collect(),
        }
    }
}

impl From<usize> for Parameter {
    fn from(o: usize) -> Self {
        Self {
            raw: o.to_string().bytes().collect(),
        }
    }
}
impl From<f64> for Parameter {
    fn from(o: f64) -> Self {
        Self {
            raw: o.to_string().bytes().collect(),
        }
    }
}
impl From<&f64> for Parameter {
    fn from(o: &f64) -> Self {
        Self {
            raw: o.to_string().bytes().collect(),
        }
    }
}

impl From<Rc<Name>> for Parameter {
    fn from(r: Rc<Name>) -> Self {
        Self {
            raw: r.to_string().bytes().collect(),
        }
    }
}
