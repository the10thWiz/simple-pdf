use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Result, Write};
use std::rc::Rc;

pub trait PDFData {
    fn write(&self, o: &mut dyn Write) -> Result<()>;
}

impl PDFData for usize {
    fn write(&self, o: &mut dyn Write) -> Result<()> {
        write!(o, "{}", self)
    }
}
impl PDFData for f64 {
    fn write(&self, o: &mut dyn Write) -> Result<()> {
        write!(o, "{}", self)
    }
}
impl PDFData for [std::string::String; 2] {
    fn write(&self, o: &mut dyn Write) -> Result<()> {
        write!(o, "[{}, {}]", self[0], self[1])
    }
}
impl<T: PDFData> PDFData for Vec<Rc<T>> {
    fn write(&self, o: &mut dyn Write) -> Result<()> {
        write!(o, "[")?;
        let mut iter = self.iter();
        if let Some(d) = iter.next() {
            d.write(o)?;
            for d in iter {
                write!(o, " ")?;
                d.write(o)?;
            }
        }
        write!(o, "]")
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct Name(String);
impl Name {
    pub fn new(s: impl Into<String>) -> Rc<Self> {
        Rc::new(Self(s.into()))
    }
}
impl From<Rc<Name>> for Name {
    fn from(n: Rc<Name>) -> Self {
        Self(n.0.clone())
    }
}
impl From<String> for Name {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "/{}", self.0)
    }
}
impl PDFData for Name {
    fn write(&self, o: &mut dyn Write) -> Result<()> {
        write!(o, "/{}", self.0)
    }
}

pub struct Dict {
    items: RefCell<HashMap<Name, Rc<dyn PDFData>>>,
}
impl Dict {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            items: RefCell::new(HashMap::new()),
        })
    }
    pub fn from_vec(v: Vec<(impl Into<Name>, Rc<dyn PDFData>)>) -> Rc<Self> {
        let mut items = HashMap::new();
        for (n, d) in v {
            items.insert(n.into(), d);
        }
        Rc::new(Self {
            items: RefCell::new(items),
        })
    }
    pub fn add_entry(&self, n: impl Into<Name>, data: Rc<dyn PDFData>) {
        self.items.borrow_mut().insert(n.into(), data);
    }
    pub fn add_optional(&self, n: impl Into<Name>, data: Option<Rc<dyn PDFData>>) {
        if let Some(data) = data {
            self.items.borrow_mut().insert(n.into(), data);
        }
    }
    pub fn is_empty(&self) -> bool {
        self.items.borrow().is_empty()
    }
}
impl PDFData for Dict {
    fn write(&self, o: &mut dyn Write) -> Result<()> {
        write!(o, "<<\n")?;
        for (k, v) in self.items.borrow().iter() {
            k.write(o)?;
            write!(o, " ")?;
            v.write(o)?;
            write!(o, "\n")?;
        }
        write!(o, ">>\n")
    }
}

pub struct Stream {
    meta: Rc<Dict>,
    data: Vec<u8>,
}

impl Stream {
    pub fn new(meta: Rc<Dict>, data: Vec<u8>) -> Rc<Self> {
        meta.add_entry("Length", Rc::new(data.len()));
        Rc::new(Self { meta, data })
    }
}

impl PDFData for Stream {
    fn write(&self, o: &mut dyn Write) -> Result<()> {
        self.meta.write(o)?;
        write!(o, "stream\n")?;
        o.write_all(&self.data)?;
        write!(o, "\nendstream\n")
    }
}
