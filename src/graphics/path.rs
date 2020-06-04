use super::{Color, Graphic, GraphicContext, Point, Rect};

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
    pub fn new() -> Self {
        Self {
            path: vec![],
            cur: Some(vec![]),
        }
    }
    pub fn from(point: impl Into<Point>) -> Self {
        Self {
            path: vec![],
            cur: Some(vec![PathPart::Start(point.into())]),
        }
    }
    pub fn move_to(mut self, point: impl Into<Point>) -> Self {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), false));
        self.cur = Some(vec![PathPart::Start(point.into())]);
        self
    }
    pub fn line_to(mut self, point: impl Into<Point>) -> Self {
        self.cur
            .as_mut()
            .unwrap()
            .push(PathPart::Line(point.into()));
        self
    }
    pub fn curve_to(mut self, p1: impl Into<Point>, p2: impl Into<Point>, p3: impl Into<Point>) -> Self {
        self.cur
            .as_mut()
            .unwrap()
            .push(PathPart::Bezier(p1.into(), p2.into(), p3.into()));
        self
    }
    pub fn curve_to_last(mut self, p2: impl Into<Point>, p3: impl Into<Point>) -> Self {
        self.cur
            .as_mut()
            .unwrap()
            .push(PathPart::BezierLast(p2.into(), p3.into()));
        self
    }
    pub fn curve_to_next(mut self, p1: impl Into<Point>, p2: impl Into<Point>) -> Self {
        self.cur
            .as_mut()
            .unwrap()
            .push(PathPart::BezierNext(p1.into(), p2.into()));
        self
    }
    pub fn rect(mut self, r: impl Into<Rect>) -> Self {
        self.path.push(SubPath::Rect(r.into()));
        self
    }
    pub fn stroke(mut self, color: Color) -> GraphicPath {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), false));
        GraphicPath {
            path: self.path,
            stroke: Some(color),
            fill: None,
            even_odd: false,
        }
    }
    pub fn fill(mut self, color: Color) -> GraphicPath {
        self.path
            .push(SubPath::Parts(self.cur.take().unwrap(), false));
        GraphicPath {
            path: self.path,
            stroke: None,
            fill: Some(color),
            even_odd: false,
        }
    }
}
pub struct GraphicPath {
    path: Vec<SubPath>,
    stroke: Option<Color>,
    fill: Option<Color>,
    even_odd: bool,
}
impl Graphic for GraphicPath {
    fn fill_color(&self) -> Option<Color> {
        self.fill
    }
    fn stroke_color(&self) -> Option<Color> {
        self.stroke
    }
    fn render(&self, g: &mut GraphicContext) {
        for subpath in &self.path {
            match subpath {
                SubPath::Parts(subpath, closed) => {
                    let mut points = subpath.iter().copied();
                    for point in points {
                        // g.command(&mut [point.into()], "l");
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
            (Some(..), Some(..)) => g.command(&mut [], "B"),
            (None, Some(..)) => g.command(&mut [], "S"),
            (Some(..), None) => g.command(&mut [], "f"),
            (None, None) => unreachable!(),
        }
    }
}
