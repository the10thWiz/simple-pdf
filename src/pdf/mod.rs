use std::cell::Cell;
use std::fmt::Debug;
use std::io::{self, Write};
use std::rc::Rc;

pub mod types;
pub use types::{Dict, Name, PDFData};

pub struct Output {
    output: Box<dyn Write>,
    pos: usize,
}

impl Output {
    pub fn new(output: Box<dyn Write>) -> Self {
        Self { output, pos: 0 }
    }
    pub fn get_pos(&self) -> usize {
        self.pos
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let tmp = self.output.write(buf);
        if let Ok(num) = tmp {
            self.pos += num;
        }
        tmp
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

pub struct CRT {
    entries: Vec<(usize, usize, usize, bool)>,
    size: usize,
}

impl CRT {
    pub fn new() -> Self {
        Self {
            entries: vec![(0, 0, 65535, true)],
            size: 0,
        }
    }
    pub fn add_entry(&mut self, offset: usize, num: usize, gen: usize, free: bool) {
        self.entries.push((offset, num, gen, free));
        if num > self.size {
            self.size = num;
        }
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
    pub fn write(mut self, o: &mut dyn Write) -> io::Result<()> {
        write!(o, "xref\n")?;
        self.entries.sort_by_key(|(_o, n, _g, _f)| *n);
        // All numbers will be used by the program
        // (And it's required by the spec)
        Self::write_part(&self.entries, 0, o)
        // let mut tmp = vec![];
        // let mut iter = self.entries.into_iter();
        // let mut last_num = 0;
        // let mut start_num = 0;
        // while let Some((offset, num, gen, free)) = iter.next() {
        //     if last_num + 1 != num && (num != 0 && last_num != 0) {
        //         // write tmp
        //         Self::write_part(&tmp, start_num, o)?;
        //         tmp = vec![];
        //         start_num = num;
        //     }
        //     tmp.push((offset, num, gen, free));
        //     last_num = num;
        // }
        // Self::write_part(&tmp, start_num, o)
    }
    fn write_part(
        entries: &Vec<(usize, usize, usize, bool)>,
        start_num: usize,
        o: &mut dyn Write,
    ) -> io::Result<()> {
        write!(o, "{} {}\n", start_num, entries.len())?;
        for (offset, _num, gen, free) in entries {
            write!(
                o,
                "{:010} {:05} {} \n",
                offset,
                gen,
                if *free { 'f' } else { 'n' }
            )?;
        }
        Ok(())
    }
}

pub enum ObjError {
    AlreadyAssigned,
    DirectObject,
}
pub trait Object: PDFData + Debug {
    fn write_obj(&self, crt: &mut CRT, out: &mut Output) -> io::Result<()>;
    fn assign_num(&self, num: usize) -> Result<(), ObjError>;
    fn is_indirect(&self) -> bool;
}
pub enum ObjRef<T: PDFData> {
    Indirect {
        num: Cell<Option<usize>>,
        gen: usize,
        data: Rc<T>,
    },
    Direct {
        data: Rc<T>,
    },
}

impl<T: PDFData> ObjRef<T> {
    pub fn new(gen: usize, data: Rc<T>) -> Rc<Self> {
        Rc::new(Self::Indirect {
            num: Cell::new(None),
            gen,
            data: data,
        })
    }
}
impl<T: PDFData> std::ops::Deref for ObjRef<T> {
    type Target = Rc<T>;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Direct { data } => &data,
            Self::Indirect { data, .. } => &data,
        }
    }
}
impl<T: PDFData> PDFData for ObjRef<T> {
    fn write(&self, o: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::Direct { data } => data.write(o),
            Self::Indirect { num, gen, .. } => {
                write!(o, "{} {} R", num.get().expect("No number assigned"), gen)
            }
        }
    }
}
impl<T: PDFData + Debug> Object for ObjRef<T> {
    fn write_obj(&self, crt: &mut CRT, out: &mut Output) -> io::Result<()> {
        match self {
            Self::Indirect { num, gen, data } => {
                crt.add_entry(out.get_pos(), num.get().expect("No num"), *gen, false);
                write!(out, "{} {} obj\n", num.get().unwrap(), gen)?;
                data.write(out)?;
                write!(out, "endobj\n")
            }
            Self::Direct { .. } => {
                panic!("Not an indirect object");
            }
        }
    }
    fn assign_num(&self, new_num: usize) -> Result<(), ObjError> {
        match self {
            Self::Direct { .. } => Err(ObjError::DirectObject),
            Self::Indirect { num, .. } => {
                if let Some(_) = num.get() {
                    Err(ObjError::AlreadyAssigned)
                } else {
                    num.set(Some(new_num));
                    Ok(())
                }
            }
        }
    }
    fn is_indirect(&self) -> bool {
        match self {
            Self::Direct { .. } => false,
            Self::Indirect { .. } => true,
        }
    }
}
impl<T: PDFData + Debug> Debug for ObjRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Direct { data } => Debug::fmt(data, f),
            Self::Indirect { num, gen, data } => {
                if let Some(d) = num.get() {
                    write!(f, "{}", d)?;
                } else {
                    write!(f, "?")?;
                }
                writeln!(f, " {} Object", gen)?;
                Debug::fmt(data, f)
            }
        }
    }
}

pub struct PDFWrite {
    objects: Vec<Rc<dyn Object>>,
    cur_num: usize,
    trailer: Trailer,
    output: Output,
}

impl PDFWrite {
    pub fn new(output: Box<dyn std::io::Write>) -> Self {
        Self {
            objects: vec![],
            // The object number 0 is reserved
            cur_num: 1,
            trailer: Trailer::new(),
            output: Output::new(output),
        }
    }
    /// Add an object the final PDF file
    ///
    /// Returns the object passed to the function
    ///
    /// # Panics
    ///
    /// panics if the object has already been added to
    /// the pdf file
    pub fn add_object(&mut self, o: Rc<dyn Object>) -> Rc<dyn Object> {
        match o.assign_num(self.cur_num) {
            Ok(()) => {
                self.objects.push(o.clone());
                self.cur_num += 1;
            }
            Err(ObjError::AlreadyAssigned) => {}
            Err(ObjError::DirectObject) => {}
        }
        for obj in o.dependent_objects() {
            self.add_object(obj);
        }
        o
    }
    /// Add an object the final PDF file, and sets
    /// the root document object to point at it.
    ///
    /// # Panics
    ///
    /// panics if the object has already been added to
    /// the pdf file
    pub fn create_root<T: 'static + PDFData>(&mut self, root: Rc<T>) -> Rc<ObjRef<T>> {
        if let Some(_) = self.trailer.root {
            panic!("An object is already root");
        }
        let o = ObjRef::new(0, root);
        self.add_object(o.clone());
        self.trailer.root = Some(o.clone());
        o
    }
    pub fn write(mut self) -> io::Result<()> {
        // let mut output = Output::new(o);
        write!(self.output, "%PDF-1.4\n%����\n")?;
        let mut crt = CRT::new();
        for obj in self.objects.iter() {
            obj.write_obj(&mut crt, &mut self.output)?;
        }
        self.trailer.size = Some(crt.get_size());
        let startxref = self.output.get_pos();
        crt.write(&mut self.output)?;
        self.trailer.write(&mut self.output)?;
        write!(self.output, "startxref\n{}\n%%EOF", startxref)
    }
}
#[derive(Debug)]
struct Trailer {
    // /Size 8
    //   /Root 1 0 R
    //   /ID [<8=1b14aafa313db63dbd6f981e49f94f4> <81b14aafa313db63dbd6f981e49f94f4>]
    size: Option<usize>,
    root: Option<Rc<dyn PDFData>>,
    info: Option<Rc<dyn PDFData>>,
    id: Option<Rc<[String; 2]>>,
}

impl Trailer {
    fn new() -> Self {
        Self {
            size: None,
            root: None,
            info: None,
            id: None,
        }
    }
}

impl PDFData for Trailer {
    fn write(&self, o: &mut dyn Write) -> io::Result<()> {
        write!(o, "trailer\n")?;
        let dict = Dict::from_vec(vec![
            ("Size", Rc::new(self.size.expect("Size not set"))),
            ("Root", self.root.clone().expect("Root not set")),
        ]);
        if let Some(info) = self.info.clone() {
            dict.add_entry("Info", info);
        }
        if let Some(id) = self.id.clone() {
            dict.add_entry("ID", id);
        }
        dict.write(o)
    }
}
