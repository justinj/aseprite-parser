use std::{
    error::Error,
    fmt::Display,
    io::{Read, Seek},
};

use constants::{ASE_USER_DATA_FLAG_HAS_COLOR, ASE_USER_DATA_FLAG_HAS_TEXT};

use crate::parser::{Parse, Parser};

mod constants;
mod metadata;
mod parser;

pub use metadata::{AsepriteFileHeader, Layer, Point, Rect, Slice, SliceKey, Tag};

struct Color(u32);

impl Color {
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color(((r as u32) << 24) + ((g as u32) << 16) + ((b as u32) << 8) + a as u32)
    }

    fn r(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    fn g(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    fn b(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    fn a(&self) -> u8 {
        self.0 as u8
    }
}

#[derive(Debug)]
pub struct Image {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

// This is the function that Aseprite uses to multiple two bytes which represent
// fractions of 255.
fn mul_un8(a: i32, b: i32) -> i32 {
    let t = a * b + 0x80;
    ((t >> 8) + t) >> 8
}

impl Image {
    fn new(width: u16, height: u16) -> Self {
        Self::new_from_data(width, height, vec![0; width as usize * height as usize * 4])
    }

    fn new_from_data(width: u16, height: u16, data: Vec<u8>) -> Self {
        assert_eq!(width as usize * height as usize * 4, data.len());
        Image {
            width,
            height,
            data,
        }
    }

    fn draw(&mut self, x: i16, y: i16, other: &Image, opacity: u8) {
        let mut idx = 0;
        let w: i16 = other.width.try_into().unwrap();
        let h: i16 = other.height.try_into().unwrap();
        for y in y..y + h {
            for x in x..x + w {
                self.draw_pixel(
                    x,
                    y,
                    Color::from_rgba(
                        other.data[idx],
                        other.data[idx + 1],
                        other.data[idx + 2],
                        other.data[idx + 3],
                    ),
                    opacity,
                );
                idx += 4;
            }
        }
    }

    fn draw_pixel(&mut self, x: i16, y: i16, source: Color, opacity: u8) {
        let x: i32 = x.into();
        let y: i32 = y.into();
        let w: i32 = self.width.into();
        let wu: usize = self.width.into();
        let h: i32 = (self.data.len() / wu).try_into().unwrap();
        if x < 0 || y < 0 || x >= w || y >= h {
            return;
        }
        let x: usize = x.try_into().expect("fits in a usize");
        let y: usize = y.try_into().expect("fits in a usize");
        let w: usize = self.width.try_into().expect("u16 fits in a usize");
        let idx: usize = (x + y * w) * 4;

        let br = self.data[idx] as i32;
        let bg = self.data[idx + 1] as i32;
        let bb = self.data[idx + 2] as i32;
        let ba = self.data[idx + 3] as i32;
        let sr = source.r() as i32;
        let sg = source.g() as i32;
        let sb = source.b() as i32;
        let opacity = opacity as i32;
        let sa = source.a() as i32;
        let sa = mul_un8(sa, opacity);

        let ra = sa + ba - mul_un8(ba, sa);

        if ra == 0 {
            self.data[idx] = 0;
            self.data[idx + 1] = 0;
            self.data[idx + 2] = 0;
            self.data[idx + 3] = 0;
        } else {
            let rr = br + (sr - br) * sa / ra;
            let rg = bg + (sg - bg) * sa / ra;
            let rb = bb + (sb - bb) * sa / ra;

            self.data[idx] = rr as u8;
            self.data[idx + 1] = rg as u8;
            self.data[idx + 2] = rb as u8;
            self.data[idx + 3] = ra as u8;
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    pub duration: u16,
    layers: Vec<Image>,
    pub image: Image,
}

#[derive(Debug)]
pub struct AsepriteFile {
    header: AsepriteFileHeader,
    layers: Vec<Layer>,
    frames: Vec<Frame>,
    tags: Vec<Tag>,
    slices: Vec<Slice>,
}

impl AsepriteFile {
    pub fn load<R: Read + Seek>(r: R) -> Result<Self, AsepriteError> {
        let mut parser = Parser::new(r);

        let header = AsepriteFileHeader::parse(&mut parser)?;
        assert_eq!(header.magic, constants::ASE_FILE_MAGIC);

        parser.seek(128)?;

        let mut file = AsepriteFile {
            header,
            layers: Vec::new(),
            frames: Vec::new(),
            tags: Vec::new(),
            slices: Vec::new(),
        };

        for _ in 0..file.header.frames {
            file.process_next_frame(&mut parser)?;
        }

        Ok(file)
    }

    fn process_next_frame<R: Read + Seek>(
        &mut self,
        parser: &mut Parser<R>,
    ) -> Result<(), AsepriteError> {
        let _size: u32 = parser.next()?;
        let magic: u16 = parser.next()?;
        let chunks: u16 = parser.next()?;
        let duration: u16 = parser.next()?;
        parser.skip(6)?;
        assert_eq!(magic, constants::ASE_FILE_FRAME_MAGIC);

        let mut frame = Frame {
            duration,
            layers: Vec::new(),
            image: Image::new(self.header.width, self.header.height),
        };

        for _ in 0..chunks {
            while self.layers.len() > frame.layers.len() {
                frame
                    .layers
                    .push(Image::new(self.header.width, self.header.height));
            }
            self.apply_chunk(&mut frame, parser)?;
        }

        for (i, l) in self.layers.iter().enumerate() {
            if l.visible() {
                frame.image.draw(0, 0, &frame.layers[i], l.opacity);
            }
        }

        self.frames.push(frame);
        Ok(())
    }

    fn apply_chunk<R: Read + Seek>(
        &mut self,
        frame: &mut Frame,
        parser: &mut Parser<R>,
    ) -> Result<(), AsepriteError> {
        let chunk_pos = parser.position();
        let chunk_size: u32 = parser.next()?;
        let chunk_type: u16 = parser.next()?;
        let chunk_size: usize = chunk_size.try_into()?;
        let chunk_end = chunk_pos + chunk_size;

        match chunk_type {
            constants::ASE_FILE_CHUNK_COLOR_PROFILE => {
                // TODO
            }
            constants::ASE_FILE_CHUNK_PALETTE => {
                // TODO
            }
            constants::ASE_FILE_CHUNK_FLI_COLOR2 => {
                // TODO
            }
            constants::ASE_FILE_CHUNK_SLICE => {
                let nkeys: u32 = parser.next()?;
                let flags: u32 = parser.next()?;
                parser.skip(4)?;
                let name: String = parser.next()?;

                let mut slice = Slice {
                    name,
                    keys: Vec::new(),
                    user_data: Default::default(),
                };

                for _ in 0..nkeys {
                    slice.keys.push(SliceKey {
                        frame: parser.next()?,
                        bounds: parser.next()?,
                        center: if flags & constants::ASE_SLICE_FLAG_HAS_CENTER_BOUNDS != 0 {
                            Some(parser.next()?)
                        } else {
                            None
                        },
                        pivot: if flags & constants::ASE_SLICE_FLAG_HAS_PIVOT_POINT != 0 {
                            Some(parser.next()?)
                        } else {
                            None
                        },
                    });
                }

                self.slices.push(slice);
            }
            constants::ASE_FILE_CHUNK_USER_DATA => {
                // These come following the slice data.
                let flags: u32 = parser.next()?;
                if flags & ASE_USER_DATA_FLAG_HAS_TEXT != 0 {
                    self.slices.last_mut().unwrap().user_data.string = parser.next()?;
                }
                if flags & ASE_USER_DATA_FLAG_HAS_COLOR != 0 {
                    self.slices.last_mut().unwrap().user_data.r = parser.next()?;
                    self.slices.last_mut().unwrap().user_data.g = parser.next()?;
                    self.slices.last_mut().unwrap().user_data.b = parser.next()?;
                    self.slices.last_mut().unwrap().user_data.a = parser.next()?;
                }
            }
            constants::ASE_FILE_CHUNK_TAGS => {
                let ntags: u16 = parser.next()?;
                parser.skip(8)?;

                for _ in 0..ntags {
                    self.tags.push(parser.next()?);
                }
            }
            constants::ASE_FILE_CHUNK_CEL => {
                let layer_index: u16 = parser.next()?;
                let x: i16 = parser.next()?;
                let y: i16 = parser.next()?;
                let opacity: u8 = parser.next()?;
                let cel_type: u16 = parser.next()?;
                parser.skip(7)?;

                match cel_type {
                    constants::ASE_FILE_COMPRESSED_CEL => {
                        let w: u16 = parser.next()?;
                        let h: u16 = parser.next()?;
                        let data = parser.next_n(chunk_end - parser.position())?;
                        // For some reason inflate uses a String instead of an Error.
                        let data = inflate::inflate_bytes_zlib(data)
                            .map_err(AsepriteError::CorruptFile)?;
                        let cel = Image::new_from_data(w, h, data);
                        frame.layers[layer_index as usize].draw(x, y, &cel, opacity);
                    }
                    constants::ASE_FILE_LINK_CEL => {
                        let linked_frame: u16 = parser.next()?;
                        let cel = &self.frames[linked_frame as usize].layers[layer_index as usize];
                        frame.layers[layer_index as usize].draw(x, y, cel, opacity);
                    }
                    ct => {
                        return Err(AsepriteError::Unimplemented(format!(
                            "unhandled cel type 0x{:x}. Please open an issue including the file you're attempting to open.",
                            ct
                        )));
                    }
                }
            }
            constants::ASE_FILE_CHUNK_LAYER => self.layers.push(parser.next()?),
            ct => {
                return Err(AsepriteError::Unimplemented(format!(
                    "unhandled chunk type 0x{:x}. Please open an issue including the file you're attempting to open.",
                    ct
                )));
            }
        }

        parser.advance_to(chunk_end)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum AsepriteError {
    Unimplemented(String),
    CorruptFile(String),
    Error(Box<dyn Error>),
}

impl Display for AsepriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsepriteError::Unimplemented(s) => write!(f, "unimplemented: {}", s),
            AsepriteError::CorruptFile(s) => write!(f, "file appears to be corrupt: {}", s),
            AsepriteError::Error(e) => e.fmt(f),
        }?;
        Ok(())
    }
}

impl<E> From<E> for AsepriteError
where
    E: 'static + Error,
{
    fn from(e: E) -> Self {
        AsepriteError::Error(Box::new(e))
    }
}

#[test]
fn test_metadata() {
    use std::fs::File;

    datadriven::walk("metadata_tests", |f| {
        let mut current_file = None;
        f.run(move |test_case| -> String {
            match test_case.directive.as_str() {
                "load" => {
                    let f = File::open(&test_case.input.trim()).unwrap();
                    current_file = Some(AsepriteFile::load(f).unwrap());
                    "ok\n".into()
                }
                "header" => {
                    format!("{:#?}\n", current_file.as_ref().unwrap().header)
                }
                "frames" => {
                    format!("{:#?}\n", current_file.as_ref().unwrap().frames)
                }
                "tags" => {
                    format!("{:#?}\n", current_file.as_ref().unwrap().tags)
                }
                "slices" => {
                    format!("{:#?}\n", current_file.as_ref().unwrap().slices)
                }
                _ => panic!("unhandled {}", test_case.directive),
            }
        })
    });
}

#[test]
fn test_files() -> Result<(), AsepriteError> {
    use std::fs::File;
    use std::path::PathBuf;

    for fname in [
        "four.ase",
        "layers1.ase",
        "layers2.ase",
        "layers3.ase",
        "layers4.ase",
        "layers5.ase",
        "invisible_layer.ase",
        "waves.ase",
        "offset.ase",
        "linked.ase",
        "linked2.ase",
        "frames.ase",
    ] {
        let mut path = PathBuf::new();
        path.push("testdata");

        let mut input_file = path.clone();
        input_file.push(fname);

        let f = File::open(input_file)?;
        let ase = AsepriteFile::load(f)?;

        for (idx, frame) in ase.frames.iter().enumerate() {
            // Load the expected.
            let mut expected = path.clone();
            expected.push(format!("{}.{}.png", fname, idx));
            let decoder = png::Decoder::new(File::open(expected)?);
            let mut reader = decoder.read_info().unwrap();
            let mut buf = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut buf).unwrap();
            let bytes = &buf[..info.buffer_size()];

            assert_eq!(bytes, frame.image.data);
        }
    }

    Ok(())
}
