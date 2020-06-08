use super::{GraphicContext, Parameter};
// use crate::pdf::{Dict, Name};
use std::cell::RefCell;
use std::rc::Rc;

pub trait Graphic: Sized {
    fn get_graphics_parameters(&self) -> &GraphicParameters;
    fn render(&self, out: &mut GraphicContext);
    fn set_fill_color(&self, color: Color) {
        self.get_graphics_parameters().fill_color(color);
    }
    fn fill_color(self, color: Color) -> Self {
        self.get_graphics_parameters().fill_color(color);
        self
    }
    fn set_stroke_color(&self, color: Color) {
        self.get_graphics_parameters().stroke_color(color);
    }
    fn stroke_color(self, color: Color) -> Self {
        self.get_graphics_parameters().stroke_color(color);
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GraphicsContextType {
    Normal,
    NoColor,
}
impl GraphicsContextType {
    fn color(&self) -> bool {
        match self {
            Self::NoColor => false,
            _ => true,
        }
    }
}
#[derive(Debug, Clone)]
pub struct GraphicParameters {
    context_type: GraphicsContextType,
    transform: (),
    clipping_path: (),
    fill: RefCell<Color>,
    stroke: RefCell<Color>,
    text_state: (),
    line_state: (),
    rendering_intent: (),
    blend_state: (),
}

impl GraphicParameters {
    pub fn with_colors(fill: Option<Color>, stroke: Option<Color>) -> Self {
        let tmp = Self {
            context_type: GraphicsContextType::Normal,
            transform: (),
            clipping_path: (),
            fill: RefCell::new(Color::default()),
            stroke: RefCell::new(Color::default()),
            text_state: (),
            line_state: (),
            rendering_intent: (),
            blend_state: (),
        };
        if let Some(color) = fill {
            tmp.fill_color(color);
        }
        if let Some(color) = stroke {
            tmp.stroke_color(color);
        }
        tmp
    }
    pub fn with_type(context_type: GraphicsContextType) -> Self {
        Self {
            context_type,
            transform: (),
            clipping_path: (),
            fill: RefCell::new(Color::default()),
            stroke: RefCell::new(Color::default()),
            text_state: (),
            line_state: (),
            rendering_intent: (),
            blend_state: (),
        }
    }
    pub fn update(ctx: &mut GraphicContext, new: &Self) {
        // Clones Rc to allow mutating the current params
        let old = ctx.current.clone();
        if old.context_type.color() {
            // Fill Color
            new.fill.borrow().write(&old.fill.borrow(), false, ctx);
            *old.fill.borrow_mut() = new.fill.borrow().clone();
            // Stroke Color
            new.stroke.borrow().write(&old.stroke.borrow(), true, ctx);
            *old.stroke.borrow_mut() = new.stroke.borrow().clone();
        }
    }
    pub fn fill_color(&self, color: Color) {
        *self.fill.borrow_mut() = color;
    }
    pub fn stroke_color(&self, color: Color) {
        *self.stroke.borrow_mut() = color;
    }
}
impl Default for GraphicParameters {
    fn default() -> Self {
        Self {
            context_type: GraphicsContextType::Normal,
            transform: (),
            clipping_path: (),
            fill: RefCell::new(Color::default()),
            stroke: RefCell::new(Color::default()),
            text_state: (),
            line_state: (),
            rendering_intent: (),
            blend_state: (),
        }
    }
}
use crate::pdf::{types::Stream, Name, ObjRef, Object};
pub struct PatternBuilder {
    graphics: GraphicContext,
}
impl PatternBuilder {
    pub fn new(colored: bool) -> Self {
        if !colored {
            Self {
                graphics: GraphicContext::with_type(GraphicsContextType::NoColor),
            }
        } else {
            Self {
                graphics: GraphicContext::with_type(GraphicsContextType::Normal),
            }
        }
    }
    pub fn add(&mut self, g: Rc<impl Graphic>) {
        self.graphics.render(g);
    }
    fn render(self) -> Color {
        let (streams, resources) = self.graphics.compile();
        if streams.len() != 1 {
            panic!("The graphics context for a pattern may only generate one stream!");
        }
        streams[0].add_entry("Type", Name::new("Pattern"));
        streams[0].add_entry("PatternType", Rc::new(1));
        Color::DeviceGray(0f64)
    }
}

#[derive(Clone, Debug)]
pub enum Color {
    DeviceGray(f64),
    DeviceRGB(f64, f64, f64),
    DeviceCMYK(f64, f64, f64, f64),
    Pattern(Rc<Name>, Rc<ObjRef<Stream>>),
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
            "CS "
        } else {
            "cs "
        }
    }
    fn set_color(stroke: bool) -> &'static str {
        if stroke {
            "SCN "
        } else {
            "scn "
        }
    }
    /// Writes self to out using cs and sc to set the color mode
    ///
    /// - prev: &Color, the current color
    /// - cs: the color space command
    /// - sc: the set color command
    /// - out: output
    fn write(&self, prev: &Color, stroke: bool, out: &mut GraphicContext) {
        if std::mem::discriminant(self) != std::mem::discriminant(prev) {
            match self {
                Self::DeviceGray(..) => out.command(
                    &mut [Name::new("DeviceGray").into()],
                    Self::set_colorspace(stroke),
                ),
                Self::DeviceRGB(..) => out.command(
                    &mut [Name::new("DeviceRGB").into()],
                    Self::set_colorspace(stroke),
                ),
                Self::DeviceCMYK(..) => out.command(
                    &mut [Name::new("DeviceCMYK").into()],
                    Self::set_colorspace(stroke),
                ),
                Self::Pattern(..) => out.command(
                    &mut [Name::new("Pattern").into()],
                    Self::set_colorspace(stroke),
                ),
            }
        }
        match self {
            Self::DeviceGray(g) => out.command(&mut [g.into()], Self::set_color(stroke)),
            Self::DeviceRGB(r, g, b) => {
                out.command(&mut [r.into(), g.into(), b.into()], Self::set_color(stroke))
            }
            Self::DeviceCMYK(c, m, y, k) => out.command(
                &mut [c.into(), m.into(), y.into(), k.into()],
                Self::set_color(stroke),
            ),
            Self::Pattern(name, obj) => {
                out.add_resource(obj.clone());
                out.command(&mut [name.clone().into()], Self::set_color(stroke))
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
