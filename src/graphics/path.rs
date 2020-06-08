use super::{Color, Graphic, GraphicContext, GraphicParameters, Point, Rect};
use std::rc::Rc;

#[derive(Clone, Debug, Copy)]
enum PathPart {
    Start(Point),
    Line(Point),
    Bezier(Point, Point, Point),
    BezierLast(Point, Point),
    BezierNext(Point, Point),
}
#[derive(Clone, Debug)]
enum SubPath {
    Parts(Vec<PathPart>, bool),
    Rect(Rect),
}

#[derive(Clone, Debug)]
pub struct Path {
    path: Vec<SubPath>,
    /// Always Some, but Option to allow .take()
    cur: Option<Vec<PathPart>>,
}

impl Path {
    /// Starts a new path
    pub fn new() -> Self {
        Self {
            path: vec![],
            cur: Some(vec![]),
        }
    }
    /// Starts a new path from the given point
    ///
    /// - point: see Point
    pub fn from(point: impl Into<Point>) -> Self {
        Self {
            path: vec![],
            cur: Some(vec![PathPart::Start(point.into())]),
        }
    }
    /// Starts a new subpath, without closing the current subpath
    ///
    /// - point: see Point
    ///
    /// This does not draw a line or curve to the point, but does not
    /// close the current subpath. See move_to_and_close for more detail
    pub fn move_to(mut self, point: impl Into<Point>) -> Self {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), false));
        self.cur = Some(vec![PathPart::Start(point.into())]);
        self
    }
    /// Starts a new subpath, while closing the current subpath
    ///
    /// - point: see Point
    ///
    /// This does not draw a line or curve to the point, and closes the
    /// current subpath. This is equavalent to `.line_to(pos).move_to(point)`
    /// where pos is the start of the current subpath
    pub fn move_to_and_close(mut self, point: impl Into<Point>) -> Self {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), true));
        self.cur = Some(vec![PathPart::Start(point.into())]);
        self
    }
    /// Adds a line to the current subpath
    ///
    /// - point: See Point for more info
    pub fn line_to(mut self, point: impl Into<Point>) -> Self {
        self.cur
            .as_mut()
            .unwrap()
            .push(PathPart::Line(point.into()));
        self
    }
    /// Adds a bezier to the current subpath
    ///
    /// - p1: See Point for more info
    /// - p2: See Point for more info
    /// - p3: See Point for more info
    ///
    /// Draws a 3rd degree bezier, using the last point, p1,
    /// p2, and p3 as it's control points.
    pub fn curve_to(
        mut self,
        p1: impl Into<Point>,
        p2: impl Into<Point>,
        p3: impl Into<Point>,
    ) -> Self {
        self.cur
            .as_mut()
            .unwrap()
            .push(PathPart::Bezier(p1.into(), p2.into(), p3.into()));
        self
    }
    /// Adds a bezier to the current subpath
    ///
    /// - p2: See Point for more info
    /// - p3: See Point for more info
    ///
    /// Draws a 3rd degree bezier, using the last point, the last point,
    /// p2, and p3 as it's control points.
    pub fn curve_to_last(mut self, p2: impl Into<Point>, p3: impl Into<Point>) -> Self {
        self.cur
            .as_mut()
            .unwrap()
            .push(PathPart::BezierLast(p2.into(), p3.into()));
        self
    }
    /// Adds a bezier to the current subpath
    ///
    /// - p1: See Point for more info
    /// - p2: See Point for more info
    ///
    /// Draws a 3rd degree bezier, using the last point, p1,
    /// p2, and p2 as it's control points.
    pub fn curve_to_next(mut self, p1: impl Into<Point>, p2: impl Into<Point>) -> Self {
        self.cur
            .as_mut()
            .unwrap()
            .push(PathPart::BezierNext(p1.into(), p2.into()));
        self
    }
    /// Adds a Rectangle to the path
    ///
    /// - r: See Rect
    ///
    /// This does not interupt or modify the current subpath,
    /// but does add a subpath. The rectangle is added before
    /// the current subpath, but that shouldn't matter to most
    /// PDF viewers
    pub fn rect(mut self, r: impl Into<Rect>) -> Self {
        self.path.push(SubPath::Rect(r.into()));
        self
    }
    /// Complete the path with a stroking operation
    ///
    /// - color: See Color
    ///
    /// Draws the current path (ending the current subpath, without closing it)
    ///
    /// # Note:
    ///
    /// Only adds the current subpath if it has more than one point. The PDF
    /// spec says that painting or clipping with a subpath that only has a
    /// single point is device dependent, so this should not cause a problem
    pub fn stroke(mut self, color: Color) -> Rc<GraphicPath> {
        if self.cur.as_ref().unwrap().len() > 1 {
            self.path
                .push(SubPath::Parts(self.cur.take().unwrap(), false));
        }
        Rc::new(GraphicPath {
            params: GraphicParameters::with_colors(None, Some(color)),
            path: self.path,
            stroke: true,
            fill: false,
            even_odd: false,
        })
    }
    /// Complete the path with a filling operation
    ///
    /// - color: See Color
    ///
    /// Draws the current path (ending the current subpath, without closing it)
    ///
    /// # Note:
    ///
    /// Only adds the current subpath if it has more than one point. The PDF
    /// spec says that painting or clipping with a subpath that only has a
    /// single point is device dependent, so this should not cause a problem
    pub fn fill(mut self, color: Color) -> Rc<GraphicPath> {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), false));
        Rc::new(GraphicPath {
            params: GraphicParameters::with_colors(Some(color), None),
            path: self.path,
            stroke: false,
            fill: true,
            even_odd: false,
        })
    }
    /// Complete the path with a stroking and filling operation
    ///
    /// - color: See Color
    ///
    /// Draws the current path (ending the current subpath, without closing it)
    ///
    /// # Note:
    ///
    /// Only adds the current subpath if it has more than one point. The PDF
    /// spec says that painting or clipping with a subpath that only has a
    /// single point is device dependent, so this should not cause a problem
    pub fn stroke_fill(mut self, stroke: Color, fill: Color) -> Rc<GraphicPath> {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), false));
        Rc::new(GraphicPath {
            params: GraphicParameters::with_colors(Some(fill), Some(stroke)),
            path: self.path,
            stroke: true,
            fill: true,
            even_odd: false,
        })
    }
    /// Complete the path with a stroking operation, using the even-odd
    /// winding rule
    ///
    /// - color: See Color
    ///
    /// Draws the current path (ending the current subpath, without closing it)
    ///
    /// # Notes:
    ///
    /// - See [Adobe's PDF 1.7 spec, 4.4.2, Even-Odd Rule](https://www.adobe.com/content/dam/acom/en/devnet/acrobat/pdfs/pdf_reference_1-7.pdf#G9.1850155)
    /// for more info
    /// - Only adds the current subpath if it has more than one point. The PDF
    /// spec says that painting or clipping with a subpath that only has a
    /// single point is device dependent, so this should not cause a problem
    pub fn stroke_even_odd(mut self, color: Color) -> Rc<GraphicPath> {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), false));
        Rc::new(GraphicPath {
            params: GraphicParameters::with_colors(None, Some(color)),
            path: self.path,
            stroke: true,
            fill: false,
            even_odd: true,
        })
    }
    /// Complete the path with a filling operation, using the even-odd
    /// winding rule
    ///
    /// - color: See Color
    ///
    /// Draws the current path (ending the current subpath, without closing it)
    ///
    /// # Notes:
    ///
    /// - See [Adobe's PDF 1.7 spec, 4.4.2, Even-Odd Rule]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/acrobat/pdfs/pdf_reference_1-7.pdf#G9.1850155)
    /// for more info
    /// - Only adds the current subpath if it has more than one point. The PDF
    /// spec says that painting or clipping with a subpath that only has a
    /// single point is device dependent, so this should not cause a problem
    pub fn fill_even_odd(mut self, color: Color) -> Rc<GraphicPath> {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), false));
        Rc::new(GraphicPath {
            params: GraphicParameters::with_colors(Some(color), None),
            path: self.path,
            stroke: false,
            fill: true,
            even_odd: true,
        })
    }
    /// Complete the path with a stroking and filling operation, using the even-odd
    /// winding rule
    ///
    /// - color: See Color
    ///
    /// Draws the current path (ending the current subpath, without closing it)
    ///
    /// # Notes:
    ///
    /// - See [Adobe's PDF 1.7 spec, 4.4.2, Even-Odd Rule]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/acrobat/pdfs/pdf_reference_1-7.pdf#G9.1850155)
    /// for more info
    /// - Only adds the current subpath if it has more than one point. The PDF
    /// spec says that painting or clipping with a subpath that only has a
    /// single point is device dependent, so this should not cause a problem
    pub fn stroke_fill_even_odd(mut self, stroke: Color, fill: Color) -> Rc<GraphicPath> {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), false));
        Rc::new(GraphicPath {
            params: GraphicParameters::with_colors(Some(fill), Some(stroke)),
            path: self.path,
            stroke: true,
            fill: true,
            even_odd: true,
        })
    }
}
#[derive(Debug)]
pub struct GraphicPath {
    params: GraphicParameters,
    path: Vec<SubPath>,
    stroke: bool,
    fill: bool,
    even_odd: bool,
}
impl Graphic for GraphicPath {
    fn get_graphics_parameters(&self) -> &GraphicParameters {
        &self.params
    }
    fn render(&self, g: &mut GraphicContext) {
        for subpath in &self.path {
            match subpath {
                SubPath::Parts(subpath, closed) => {
                    for point in subpath.iter().copied() {
                        match point {
                            PathPart::Start(p) => g.command(&mut [p.into()], "m"),
                            PathPart::Line(p) => g.command(&mut [p.into()], "l"),
                            PathPart::Bezier(p1, p2, p3) => {
                                g.command(&mut [p1.into(), p2.into(), p3.into()], "c")
                            }
                            PathPart::BezierLast(p1, p2) => {
                                g.command(&mut [p1.into(), p2.into()], "v")
                            }
                            PathPart::BezierNext(p1, p2) => {
                                g.command(&mut [p1.into(), p2.into()], "y")
                            }
                        }
                    }
                    if *closed {
                        g.command(&mut [], "h");
                    }
                }
                SubPath::Rect(r) => g.command(&mut [(*r).into()], "re"),
            }
        }
        match (self.fill, self.stroke) {
            (true, true) => {
                if self.even_odd {
                    g.command(&mut [], "B*")
                } else {
                    g.command(&mut [], "B")
                }
            }
            (false, true) => g.command(&mut [], "S"),
            (true, false) => {
                if self.even_odd {
                    g.command(&mut [], "f*")
                } else {
                    g.command(&mut [], "f")
                }
            }
            (false, false) => unreachable!(),
        }
    }
}
