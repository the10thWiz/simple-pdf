use crate::pdf::{Dict, Name, Object};
use std::collections::LinkedList;
use std::rc::Rc;
pub mod path;
pub use path::Path;
pub mod text;
pub use text::{Font, Text};

pub struct GraphicState {
    transform: (),
    clipping_path: (),
    fill: Color,
    stroke: Color,
    text_state: (),
    line_state: (),
    rendering_intent: (),
    blend_state: (),
}
impl GraphicState {
    fn new() -> Self {
        GraphicState {
            transform: (),
            clipping_path: (),
            fill: Color::default(),
            stroke: Color::default(),
            text_state: (),
            line_state: (),
            rendering_intent: (),
            blend_state: (),
        }
    }
    fn update(&mut self, stream: &mut Vec<u8>, object: Rc<impl Graphic>) {
        if let Some(fill) = object.fill_color() {
            fill.write(&self.fill, false, stream);
            self.fill = fill;
        }
        if let Some(stroke) = object.stroke_color() {
            stroke.write(&self.stroke, true, stream);
            self.stroke = stroke;
        }
    }
}

pub struct GraphicContext {
    // Mutable state
    current: GraphicState,
    stack: LinkedList<GraphicState>,
    stream: Vec<u8>,
    // Resource Dict
    resources: Rc<Dict>,
    fonts: Rc<Dict>,
    external_resources: Vec<Rc<Object>>,
}
impl GraphicContext {
    pub fn new() -> Self {
        Self {
            current: GraphicState::new(),
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
        self.current.update(&mut self.stream, object.clone());
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
    fn add_font(&mut self, f: Rc<text::Font>) {
        self.fonts.add_entry(f.name(), f.object());
        self.external_resources.push(f.object());
    }
    pub fn compile(
        self,
        write: &mut crate::pdf::PDFWrite,
    ) -> (Vec<Rc<crate::pdf::Object>>, Rc<crate::pdf::Dict>) {
        if !self.fonts.is_empty() {
            self.resources.add_entry("Font", self.fonts);
        }

        let streams = vec![crate::pdf::Object::new(
            0,
            crate::pdf::types::Stream::new(crate::pdf::Dict::new(), self.stream),
        )];
        for obj in streams.iter().cloned() {
            write.add_object(obj);
        }
        for obj in self.external_resources {
            write.add_object(obj);
        }
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

impl From<Point> for Parameter {
    fn from(p: Point) -> Self {
        Self {
            raw: [
                p.0.to_string().bytes(),
                " ".bytes(),
                p.1.to_string().bytes(),
            ]
            .iter_mut()
            .flatten()
            .collect(),
        }
    }
}

impl From<Rect> for Parameter {
    fn from(r: Rect) -> Self {
        Self {
            raw: [
                r.0.to_string().bytes(),
                " ".bytes(),
                r.1.to_string().bytes(),
                " ".bytes(),
                r.2.to_string().bytes(),
                " ".bytes(),
                r.3.to_string().bytes(),
            ]
            .iter_mut()
            .flatten()
            .collect(),
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

pub trait Graphic {
    fn fill_color(&self) -> Option<Color>;
    fn stroke_color(&self) -> Option<Color>;
    fn render(&self, out: &mut GraphicContext);
}

#[derive(Clone, Copy, Debug)]
pub enum Color {
    DeviceGray(f64),
    DeviceRGB(f64, f64, f64),
    DeviceCMYK(f64, f64, f64, f64),
}
impl Color {
    pub fn default() -> Self {
        Self::DeviceGray(0f64)
    }
    pub fn red() -> Self {
        Self::DeviceRGB(1f64, 0f64, 0f64)
    }
    fn set_colorspace(stroke: bool) -> &'static str {
        if stroke {
            "CS"
        } else {
            "cs"
        }
    }
    fn set_color(stroke: bool) -> &'static str {
        if stroke {
            "SCN"
        } else {
            "scn"
        }
    }
    /// Writes self to out using cs and sc to set the color mode
    ///
    /// - prev: &Color, the current color
    /// - cs: the color space command
    /// - sc: the set color command
    /// - out: output
    fn write(&self, prev: &Color, stroke: bool, out: &mut Vec<u8>) {
        if std::mem::discriminant(self) != std::mem::discriminant(prev) {
            match self {
                Self::DeviceGray(..) => {
                    out.extend("/DeviceGray ".bytes());
                    out.extend(Self::set_colorspace(stroke).bytes());
                }
                Self::DeviceRGB(..) => {
                    out.extend("/DeviceRGB ".bytes());
                    out.extend(Self::set_colorspace(stroke).bytes());
                }
                Self::DeviceCMYK(..) => {
                    out.extend("/DeviceCMYK ".bytes());
                    out.extend(Self::set_colorspace(stroke).bytes());
                }
            }
        }
        match self {
            Self::DeviceGray(g) => {
                out.extend(g.to_string().bytes());
                out.push(' ' as u8);
                out.extend(Self::set_color(stroke).bytes());
            }
            Self::DeviceRGB(r, g, b) => {
                out.extend(
                    [
                        r.to_string().bytes(),
                        " ".bytes(),
                        g.to_string().bytes(),
                        " ".bytes(),
                        b.to_string().bytes(),
                        " ".bytes(),
                    ]
                    .iter_mut()
                    .flatten(),
                );
                out.extend(Self::set_color(stroke).bytes());
            }
            Self::DeviceCMYK(c, m, y, k) => {
                out.extend(
                    [
                        c.to_string().bytes(),
                        " ".bytes(),
                        m.to_string().bytes(),
                        " ".bytes(),
                        y.to_string().bytes(),
                        " ".bytes(),
                        k.to_string().bytes(),
                        " ".bytes(),
                    ]
                    .iter_mut()
                    .flatten(),
                );
                out.extend(Self::set_color(stroke).bytes());
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point(f64, f64);

impl From<(f64, f64)> for Point {
    fn from(o: (f64, f64)) -> Self {
        Self(o.0, o.1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect(f64, f64, f64, f64);
impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self(x, y, w, h)
    }
    pub fn as_data(&self) -> Rc<Vec<Rc<f64>>> {
        Rc::new(vec![
            Rc::new(self.0),
            Rc::new(self.1),
            Rc::new(self.2),
            Rc::new(self.3),
        ])
    }
}
impl From<(f64, f64, f64, f64)> for Rect {
    fn from(o: (f64, f64, f64, f64)) -> Self {
        Self(o.0, o.1, o.2, o.3)
    }
}
