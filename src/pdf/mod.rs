use std::cell::{Cell, RefCell};
use std::io::{self, Write};
use std::rc::Rc;

pub mod types;
pub use types::{Dict, Name, PDFData};

struct Output<T: Write> {
    output: T,
    pos: usize,
}

impl<T: Write> Output<T> {
    pub fn new(output: T) -> Self {
        Self { output, pos: 0 }
    }
    pub fn get_pos(&self) -> usize {
        self.pos
    }
}

impl<T: Write> Write for Output<T> {
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

struct CRT {
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

pub enum Object {
    Indirect {
        num: Cell<Option<usize>>,
        gen: usize,
        data: RefCell<Option<Rc<dyn PDFData>>>,
    },
    Direct {
        data: RefCell<Option<Rc<dyn PDFData>>>,
    },
}
enum ObjError {
    AlreadyAssigned,
    DirectObject,
}

impl Object {
    pub fn new(gen: usize, data: Rc<dyn PDFData>) -> Rc<Self> {
        Rc::new(Self::Indirect {
            num: Cell::new(None),
            gen,
            data: RefCell::new(Some(data)),
        })
    }
    pub fn empty(gen: usize) -> Rc<Self> {
        Rc::new(Self::Indirect {
            num: Cell::new(None),
            gen,
            data: RefCell::new(None),
        })
    }
    pub fn assign(&self, new_data: Rc<dyn PDFData>) {
        match self {
            Self::Indirect { data, .. } => {
                let mut it = data.borrow_mut();
                if let Some(_) = it.as_ref() {
                    panic!("Already assigned some data");
                }
                *it = Some(new_data);
            }
            Self::Direct { data } => {
                let mut it = data.borrow_mut();
                if let Some(_) = it.as_ref() {
                    panic!("Already assigned some data");
                }
                *it = Some(new_data);
            }
        }
    }
    fn write_obj<T: Write>(&self, crt: &mut CRT, out: &mut Output<T>) -> io::Result<()> {
        match self {
            Self::Indirect { num, gen, data } => {
                crt.add_entry(out.get_pos(), num.get().expect("No num"), *gen, false);
                write!(out, "{} {} obj\n", num.get().unwrap(), gen)?;
                data.borrow()
                    .as_ref()
                    .expect("Object has no data")
                    .write(out)?;
                write!(out, "endobj\n")
            }
            Self::Direct { .. } => {
                panic!("Not an indirect object");
            }
        }
    }
    fn assign_num(&self, new_num: usize) -> Result<(), ObjError> {
        match self {
            Self::Indirect { num, .. } => {
                if let Some(_) = num.get() {
                    return Err(ObjError::AlreadyAssigned);
                }
                num.set(Some(new_num));
            }
            Self::Direct { .. } => {
                return Err(ObjError::DirectObject);
            }
        }
        Ok(())
    }
}
impl PDFData for Object {
    fn write(&self, o: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::Indirect { num, gen, .. } => write!(
                o,
                "{} {} R",
                num.get().expect("Object not added to pdf"),
                gen
            ),
            Self::Direct { data } => data.borrow().as_ref().expect("Object has no data").write(o),
        }
    }
}

pub struct PDFWrite {
    objects: Vec<Rc<Object>>,
    cur_num: usize,
    trailer: Trailer,
}

impl PDFWrite {
    pub fn new() -> Self {
        Self {
            objects: vec![],
            // The object number 0 is reserved
            cur_num: 1,
            trailer: Trailer::new(),
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
    pub fn add_object(&mut self, o: Rc<Object>) -> Rc<Object> {
        match o.assign_num(self.cur_num) {
            Ok(()) => {
                self.objects.push(o.clone());
                self.cur_num += 1;
            }
            Err(ObjError::AlreadyAssigned) => {}
            Err(ObjError::DirectObject) => {}
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
    pub fn set_root(&mut self, o: Rc<Object>) {
        match o.as_ref() {
            Object::Indirect { .. } => {
                if let Some(_) = self.trailer.root {
                    panic!("An object is already root");
                }
                self.add_object(o.clone());
                self.trailer.root = Some(o);
            }
            _ => panic!("Root must be indirect object"),
        }
    }
    pub fn write(mut self, o: &mut dyn Write) -> io::Result<()> {
        let mut output = Output::new(o);
        write!(output, "%PDF-1.4\n%����\n")?;
        let mut crt = CRT::new();
        for obj in self.objects.iter() {
            obj.write_obj(&mut crt, &mut output)?;
        }
        self.trailer.size = Some(crt.get_size());
        let startxref = output.get_pos();
        crt.write(&mut output)?;
        self.trailer.write(&mut output)?;
        write!(output, "startxref\n{}\n%%EOF", startxref)
    }
}

struct Trailer {
    // /Size 8
    //   /Root 1 0 R
    //   /ID [<8=1b14aafa313db63dbd6f981e49f94f4> <81b14aafa313db63dbd6f981e49f94f4>]
    size: Option<usize>,
    root: Option<Rc<Object>>,
    info: Option<Rc<Object>>,
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
