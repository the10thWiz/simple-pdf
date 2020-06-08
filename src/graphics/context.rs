use crate::pdf::{Dict, Name};
use std::rc::Rc;

pub struct GraphicContext {
    // Output stream
    stream: Vec<u8>,
    // Resource Dict
    resources: Rc<Dict>,
    fonts: Rc<Dict>,
    // external_resources: Vec<Rc<Object>>,
}

impl GraphicContext {
    pub fn new() -> Self {
        Self {
            stream: vec![],
            resources: Dict::from_vec(vec![(
                "ProcSet",
                Rc::new(vec![Name::new("PDF"), Name::new("Text")]),
            )]),
            fonts: Dict::new(),
            // external_resources: vec![],
        }
    }
    /// Renders a object to the context
    ///
    /// Uses the current graphics state to control the rendering process
    pub fn render(&mut self, object: ()) {
        unimplemented!()
    }
    /// TODO: Write methods to set graphics paramters
    ///
    /// Color, font, line width, etc.
    pub fn set_parameter(&mut self, parameter: ()) {
        unimplemented!()
    }
}
