use super::{Color, Graphic, GraphicContext, Point};
use crate::pdf::{Dict, Name, ObjRef, PDFData};
use std::io::{self, Write};
use std::rc::Rc;

enum Update<T> {
    New(T),
    Old(T),
}
impl<T> Update<T> {
    fn unwrap(&self) -> &T {
        match self {
            Self::New(d) => d,
            Self::Old(d) => d,
        }
    }
}
impl<T: Clone + PartialEq<T>> Update<T> {
    fn update(&mut self) -> Option<T> {
        match self {
            Self::New(d) => {
                let tmp = Some(d.clone());
                *self = Self::Old(d.clone());
                tmp
            }
            Self::Old(..) => None,
        }
    }
    fn replace(&mut self, new: T) {
        match self {
            Self::New(..) => *self = Self::New(new),
            Self::Old(d) => {
                if d != &new {
                    *self = Self::New(new);
                }
            }
        }
    }
}
impl<T: PartialEq<T>> PartialEq for Update<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::New(s), Self::New(o)) => s == o,
            (Self::Old(s), Self::Old(o)) => s == o,
            _ => false,
        }
    }
}

#[derive(PartialEq)]
struct TextPart {
    text: String,
    font: Option<(Rc<Font>, f64)>,
    pos: Option<Point>,
}

#[derive(PartialEq)]
pub struct Text {
    parts: Vec<TextPart>,
    font: Update<(Rc<Font>, f64)>,
    pos: Update<Point>,
}

impl Text {
    pub fn new(font: Rc<Font>, size: f64) -> Self {
        Self {
            parts: vec![],
            font: Update::New((font, size)),
            pos: Update::New((0f64, 0f64).into()),
        }
    }
    pub fn move_to(mut self, p: impl Into<Point>) -> Self {
        self.pos.replace(p.into());
        self
    }
    pub fn with_font(mut self, font: Rc<Font>, size: f64) -> Self {
        self.font.replace((font, size));
        self
    }
    pub fn font_size(mut self, size: f64) -> Self {
        self.font.replace((self.font.unwrap().0.clone(), size));
        self
    }
    pub fn text(mut self, p: impl Into<String>) -> Self {
        self.parts.push(TextPart {
            text: p.into(),
            font: self.font.update(),
            pos: self.pos.update(),
        });
        self
    }
    pub fn fill(mut self, color: Color) -> GraphicText {
        GraphicText {
            parts: self.parts,
            fill: Some(color),
            stroke: None,
        }
    }
}

// TODO:
// Use utf-16 with BOM '254u8', '255u8'
// .encode_utf16() for iter, flat map to spilt bytes

pub struct GraphicText {
    parts: Vec<TextPart>,
    stroke: Option<Color>,
    fill: Option<Color>,
}

impl Graphic for GraphicText {
    fn fill_color(&self) -> Option<Color> {
        self.fill
    }
    fn stroke_color(&self) -> Option<Color> {
        self.stroke
    }
    fn render(&self, out: &mut GraphicContext) {
        out.command(&mut [], "BT");
        for part in self.parts.iter() {
            if let Some((font, size)) = &part.font {
                out.add_font(font.clone());
                out.command(&mut [font.name.clone().into(), (*size).into()], "Tf");
            }
            if let Some(pos) = part.pos {
                out.command(&mut [pos.into()], "Td");
            }
            out.command(&mut [(&part.text).into()], "Tj");
        }
        out.command(&mut [], "ET");
    }
}

enum FontType {
    Type1,
    MMType1,
}
impl FontType {
    fn to_name(&self) -> Rc<Name> {
        match self {
            Self::Type1 => Name::new("Type1"),
            Self::MMType1 => Name::new("MMType1"),
        }
    }
}
pub struct FontObject {
    // /Type /Font
    subtype: FontType,
    base_font: Rc<Name>,
    // optional only for standard 14 fonts
    first_char: Option<ObjRef<usize>>,
    last_char: Option<ObjRef<usize>>,
    widths: Option<ObjRef<usize>>,
    font_descriptor: Option<ObjRef<usize>>,
    // Fully optional
    encoding: Option<ObjRef<usize>>,
    to_unicode: Option<ObjRef<usize>>,
}
impl FontObject {
    fn new(
        subtype: FontType,
        base_font: Rc<Name>,
        first_char: Option<ObjRef<usize>>,
        last_char: Option<ObjRef<usize>>,
        widths: Option<ObjRef<usize>>,
        font_descriptor: Option<ObjRef<usize>>,
        encoding: Option<ObjRef<usize>>,
        to_unicode: Option<ObjRef<usize>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            subtype,
            base_font,
            first_char,
            last_char,
            widths,
            font_descriptor,
            encoding,
            to_unicode,
        })
    }
}
impl PDFData for FontObject {
    fn write(&self, o: &mut dyn Write) -> io::Result<()> {
        Dict::from_vec(vec![
            ("Type", Name::new("Font")),
            ("Subtype", self.subtype.to_name()),
            ("BaseFont", self.base_font.clone()),
        ])
        .write(o)
    }
}

pub struct Font {
    name: Rc<Name>,
    object: Rc<ObjRef<FontObject>>,
}
impl Font {
    /// Internal Object for constructing pdf
    pub fn name(&self) -> Rc<Name> {
        self.name.clone()
    }
    /// Internal Object for constructing pdf
    pub fn object(&self) -> Rc<ObjRef<FontObject>> {
        self.object.clone()
    }
    /// One of the 14 standard fonts
    pub fn times_new_roman() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("timesroman"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Times-Roman"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn helvetica() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("helvetica"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Helvetica"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn courier() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("courier"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Courier"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn symbol() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("symbol"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Symbol"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn times_bold() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("timesbold"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Times−Bold"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn helvetica_bold() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("helveticabold"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("helveticabold"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn courier_bold() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("courierbold"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Courier−Bold"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn zapf_dingbats() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("zapfdingbats"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("ZapfDingbats"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn times_italic() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("timesitalic"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Times−Italic"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn helvetica_oblique() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("helveticaoblique"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("helveticaoblique"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn courier_oblique() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("courieroblique"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Courier−Oblique"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn times_bold_italic() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("timesbolditalic"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Times−BoldItalic"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn helvetica_bold_oblique() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("helveticaboldoblique"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Helvetica−BoldOblique"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
    /// One of the 14 standard fonts
    pub fn courier_bold_oblique() -> Rc<Self> {
        Rc::new(Self {
            name: Name::new("courierboldoblique"),
            object: ObjRef::new(
                0,
                FontObject::new(
                    FontType::Type1,
                    Name::new("Courier−BoldOblique"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            ),
        })
    }
}
impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

mod pdf_doc_encode {
    #[allow(unused)]
    fn decode(c: u8) {
        match c {
            0x00 => (), // U+0000
            0x01 => (), // U+0001
            0x02 => (), // U+0002
            0x03 => (), // U+0003
            0x04 => (), // U+0004
            0x05 => (), // U+0005
            0x06 => (), // U+0006
            0x07 => (), // U+0007
            0x08 => (), // U+0008
            0x09 => (), // U+0009
            0x0a => (), // U+000A
            0x0b => (), // U+000B
            0x0c => (), // U+000C
            0x0d => (), // U+000D
            0x0e => (), // U+000E
            0x0f => (), // U+000F
            0x10 => (), // U+0010
            0x11 => (), // U+0011
            0x12 => (), // U+0012
            0x13 => (), // U+0013
            0x14 => (), // U+0014
            0x15 => (), // U+0015
            0x16 => (), // U+0017
            0x17 => (), // U+0017
            0x18 => (), // U+02D8
            0x19 => (), // U+02C7
            0x1a => (), // U+02C6
            0x1b => (), // U+02D9
            0x1c => (), // U+02DD
            0x1d => (), // U+02DB
            0x1e => (), // U+02DA
            0x1f => (), // U+02DC
            0x20 => (), // U+0020
            0x21 => (), // U+0021
            0x22 => (), // U+0022
            0x23 => (), // U+0023
            0x24 => (), // U+0024
            0x25 => (), // U+0025
            0x26 => (), // U+0026
            0x27 => (), // U+0027
            0x28 => (), // U+0028
            0x29 => (), // U+0029
            0x2a => (), // U+002A
            0x2b => (), // U+002B
            0x2c => (), // U+002C
            0x2d => (), // U+002D
            0x2e => (), // U+002E
            0x2f => (), // U+002F
            0x30 => (), // U+0030
            0x31 => (), // U+0031
            0x32 => (), // U+0032
            0x33 => (), // U+0033
            0x34 => (), // U+0034
            0x35 => (), // U+0035
            0x36 => (), // U+0036
            0x37 => (), // U+0037
            0x38 => (), // U+0038
            0x39 => (), // U+0039
            0x3a => (), // U+003A
            0x3b => (), // U+003B
            0x3c => (), // U+003C
            0x3d => (), // U+003D
            0x3e => (), // U+003E
            0x3f => (), // U+003F
            0x40 => (), // U+0040
            0x41 => (), // U+0041
            0x42 => (), // U+0042
            0x43 => (), // U+0043
            0x44 => (), // U+0044
            0x45 => (), // U+0045
            0x46 => (), // U+0046
            0x47 => (), // U+0047
            0x48 => (), // U+0048
            0x49 => (), // U+0049
            0x4a => (), // U+004A
            0x4b => (), // U+004B
            0x4c => (), // U+004C
            0x4d => (), // U+004D
            0x4e => (), // U+004E
            0x4f => (), // U+004F
            0x50 => (), // U+0050
            0x51 => (), // U+0051
            0x52 => (), // U+0052
            0x53 => (), // U+0053
            0x54 => (), // U+0054
            0x55 => (), // U+0055
            0x56 => (), // U+0056
            0x57 => (), // U+0057
            0x58 => (), // U+0058
            0x59 => (), // U+0059
            0x5a => (), // U+005A
            0x5b => (), // U+005B
            0x5c => (), // U+005C
            0x5d => (), // U+005D
            0x5e => (), // U+005E
            0x5f => (), // U+005F
            0x60 => (), // U+0060
            0x61 => (), // U+0061
            0x62 => (), // U+0062
            0x63 => (), // U+0063
            0x64 => (), // U+0064
            0x65 => (), // U+0065
            0x66 => (), // U+0066
            0x67 => (), // U+0067
            0x68 => (), // U+0068
            0x69 => (), // U+0069
            0x6a => (), // U+006A
            0x6b => (), // U+006B
            0x6c => (), // U+006C
            0x6d => (), // U+006D
            0x6e => (), // U+006E
            0x6f => (), // U+006F
            0x70 => (), // U+0070
            0x71 => (), // U+0071
            0x72 => (), // U+0072
            0x73 => (), // U+0073
            0x74 => (), // U+0074
            0x75 => (), // U+0075
            0x76 => (), // U+0076
            0x77 => (), // U+0077
            0x78 => (), // U+0078
            0x79 => (), // U+0079
            0x7a => (), // U+007A
            0x7b => (), // U+007B
            0x7c => (), // U+007C
            0x7d => (), // U+007D
            0x7e => (), // U+007E
            0x7f => panic!("Undefined"),
            0x80 => (), // U+2022
            0x81 => (), // U+2020
            0x82 => (), // U+2021
            0x83 => (), // U+2026
            0x84 => (), // U+2014
            0x85 => (), // U+2013
            0x86 => (), // U+0192
            0x87 => (), // U+2044
            0x88 => (), // U+2039
            0x89 => (), // U+203A
            0x8a => (), // U+2212
            0x8b => (), // U+2030
            0x8c => (), // U+201E
            0x8d => (), // U+201C
            0x8e => (), // U+201D
            0x8f => (), // U+2018
            0x90 => (), // U+2019
            0x91 => (), // U+201A
            0x92 => (), // U+2122
            0x93 => (), // U+FB01
            0x94 => (), // U+FB02
            0x95 => (), // U+0141
            0x96 => (), // U+0152
            0x97 => (), // U+0160
            0x98 => (), // U+0178
            0x99 => (), // U+017D
            0x9a => (), // U+0131
            0x9b => (), // U+0142
            0x9c => (), // U+0153
            0x9d => (), // U+0161
            0x9e => (), // U+017E
            0x9f => panic!("Undefined"),
            0xa0 => (), // U+20AC
            0xa1 => (), // U+00A1
            0xa2 => (), // U+00A2
            0xa3 => (), // U+00A3
            0xa4 => (), // U+00A4
            0xa5 => (), // U+00A5
            0xa6 => (), // U+00A6
            0xa7 => (), // U+00A7
            0xa8 => (), // U+00A8
            0xa9 => (), // U+00A9
            0xaa => (), // U+00AA
            0xab => (), // U+00AB
            0xac => (), // U+00AC
            0xad => panic!("Undefined"),
            0xae => (), // U+00AE
            0xaf => (), // U+00AF
            0xb0 => (), // U+00B0
            0xb1 => (), // U+00B1
            0xb2 => (), // U+00B2
            0xb3 => (), // U+00B3
            0xb4 => (), // U+00B4
            0xb5 => (), // U+00B5
            0xb6 => (), // U+00B6
            0xb7 => (), // U+00B7
            0xb8 => (), // U+00B8
            0xb9 => (), // U+00B9
            0xba => (), // U+00BA
            0xbb => (), // U+00BB
            0xbc => (), // U+00BC
            0xbd => (), // U+00BD
            0xbe => (), // U+00BE
            0xbf => (), // U+00BF
            0xc0 => (), // U+00C0
            0xc1 => (), // U+00C1
            0xc2 => (), // U+00C2
            0xc3 => (), // U+00C3
            0xc4 => (), // U+00C4
            0xc5 => (), // U+00C5
            0xc6 => (), // U+00C6
            0xc7 => (), // U+00C7
            0xc8 => (), // U+00C8
            0xc9 => (), // U+00C9
            0xca => (), // U+00CA
            0xcb => (), // U+00CB
            0xcc => (), // U+00CC
            0xcd => (), // U+00CD
            0xce => (), // U+00CE
            0xcf => (), // U+00CF
            0xd0 => (), // U+00D0
            0xd1 => (), // U+00D1
            0xd2 => (), // U+00D2
            0xd3 => (), // U+00D3
            0xd4 => (), // U+00D4
            0xd5 => (), // U+00D5
            0xd6 => (), // U+00D6
            0xd7 => (), // U+00D7
            0xd8 => (), // U+00D8
            0xd9 => (), // U+00D9
            0xda => (), // U+00DA
            0xdb => (), // U+00DB
            0xdc => (), // U+00DC
            0xdd => (), // U+00DD
            0xde => (), // U+00DE
            0xdf => (), // U+00DF
            0xe0 => (), // U+00E0
            0xe1 => (), // U+00E1
            0xe2 => (), // U+00E2
            0xe3 => (), // U+00E3
            0xe4 => (), // U+00E4
            0xe5 => (), // U+00E5
            0xe6 => (), // U+00E6
            0xe7 => (), // U+00E7
            0xe8 => (), // U+00E8
            0xe9 => (), // U+00E9
            0xea => (), // U+00EA
            0xeb => (), // U+00EB
            0xec => (), // U+00EC
            0xed => (), // U+00ED
            0xee => (), // U+00EE
            0xef => (), // U+00EF
            0xf0 => (), // U+00F0
            0xf1 => (), // U+00F1
            0xf2 => (), // U+00F2
            0xf3 => (), // U+00F3
            0xf4 => (), // U+00F4
            0xf5 => (), // U+00F5
            0xf6 => (), // U+00F6
            0xf7 => (), // U+00F7
            0xf8 => (), // U+00F8
            0xf9 => (), // U+00F9
            0xfa => (), // U+00FA
            0xfb => (), // U+00FB
            0xfc => (), // U+00FC
            0xfd => (), // U+00FD
            0xfe => (), // U+00FE
            0xff => (), // U+00FF
        }
    }
}
