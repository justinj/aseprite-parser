use std::{
    error::Error,
    fmt::Display,
    io::{BufReader, Read, Seek, SeekFrom},
    mem::size_of,
};

mod constants;

trait Parse: Sized {
    fn parse<R: Read + Seek>(p: &mut Parser<R>) -> Result<Self, AsepriteError>;
}

macro_rules! impl_parse {
    ($type_name:ty) => {
        impl Parse for $type_name {
            fn parse<R: Read + Seek>(p: &mut Parser<R>) -> Result<Self, AsepriteError> {
                let n = size_of::<Self>();
                let next_n = p.next_n(n)?;
                Ok(Self::from_le_bytes(next_n.try_into()?))
            }
        }
    };
}

impl_parse!(u8);
impl_parse!(u16);
impl_parse!(u32);
impl_parse!(u64);
impl_parse!(i8);
impl_parse!(i16);
impl_parse!(i32);
impl_parse!(i64);

impl Parse for String {
    fn parse<R: Read + Seek>(p: &mut Parser<R>) -> Result<Self, AsepriteError> {
        let len = u16::parse(p)?.try_into()?;
        Ok(String::from_utf8(p.next_n(len)?.to_vec())?)
    }
}

#[derive(Debug)]
struct Parser<R>
where
    R: Read,
{
    buf: Vec<u8>,
    reader: BufReader<R>,
    pos: usize,
}

impl<R> Parser<R>
where
    R: Read + Seek,
{
    fn new(r: R) -> Self {
        Parser {
            buf: Vec::new(),
            reader: BufReader::new(r),
            pos: 0,
        }
    }

    fn seek(&mut self, n: u64) -> Result<(), AsepriteError> {
        self.reader.seek(SeekFrom::Start(n))?;
        Ok(())
    }

    fn next_n(&mut self, n: usize) -> Result<&[u8], AsepriteError> {
        self.pos += n;
        self.buf.clear();
        self.buf.extend((0..n).map(|_| 0));
        self.reader.read_exact(&mut self.buf)?;
        Ok(&self.buf)
    }

    fn next<P: Parse>(&mut self) -> Result<P, AsepriteError> {
        P::parse(self)
    }

    fn skip(&mut self, n: usize) -> Result<(), AsepriteError> {
        self.next_n(n)?;
        Ok(())
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn advance_to(&mut self, n: usize) -> Result<(), AsepriteError> {
        if n < self.pos {
            return Err(AsepriteError::CorruptFile(
                "cannot advance past current position".into(),
            ));
        }
        let extra = n - self.pos;
        let _ = self.next_n(extra)?;
        Ok(())
    }
}

#[derive(Debug)]
struct AsepriteFileHeader {
    size: u32,
    magic: u16,
    frames: u16,
    width: u16,
    height: u16,
    depth: u16,
    flags: u32,
    speed: u16,
    next: u32,
    frit: u32,
    transparent_index: u32,
    ignore0: u8,
    ignore1: u8,
    ignore2: u8,
    ncolors: u16,
    pixel_width: u8,
    pixel_height: u8,
    grid_x: i16,
    grid_y: i16,
    grid_width: u16,
    grid_height: u16,
}

impl Parse for AsepriteFileHeader {
    fn parse<R>(p: &mut Parser<R>) -> Result<Self, AsepriteError>
    where
        R: Read + Seek,
    {
        Ok(AsepriteFileHeader {
            size: p.next()?,
            magic: p.next()?,
            frames: p.next()?,
            width: p.next()?,
            height: p.next()?,
            depth: p.next()?,
            flags: p.next()?,
            speed: p.next()?,
            next: p.next()?,
            frit: p.next()?,
            transparent_index: p.next()?,
            ignore0: p.next()?,
            ignore1: p.next()?,
            ignore2: p.next()?,
            ncolors: p.next()?,
            pixel_width: p.next()?,
            pixel_height: p.next()?,
            grid_x: p.next()?,
            grid_y: p.next()?,
            grid_width: p.next()?,
            grid_height: p.next()?,
        })
    }
}

#[derive(Debug)]
struct Image {
    width: u16,
    height: u16,
    data: Vec<u8>,
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
                    other.data[idx],
                    other.data[idx + 1],
                    other.data[idx + 2],
                    other.data[idx + 3],
                    opacity,
                );
                idx += 4;
            }
        }
    }

    fn draw_pixel(&mut self, x: i16, y: i16, sr: u8, sg: u8, sb: u8, sa: u8, opacity: u8) {
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
        let sr = sr as i32;
        let sg = sg as i32;
        let sb = sb as i32;
        let opacity = opacity as i32;
        let sa = sa as i32;
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
struct Frame {
    duration: u16,
    layers: Vec<Image>,
    image: Image,
}

#[derive(Debug)]
struct Layer {
    flags: u16,
    layer_type: u16,
    child_level: u16,
    default_width: u16,
    default_height: u16,
    blend_mode: u16,
    opacity: u8,
    name: String,
}

fn mul_un8(a: i32, b: i32) -> i32 {
    let t = a * b + 0x80;
    ((t >> 8) + t) >> 8
}

impl Layer {
    fn visible(&self) -> bool {
        self.flags & constants::LAYER_VISIBLE != 0
    }
}

#[derive(Debug)]
struct ChunkHeader {}

#[derive(Debug)]
struct AsepriteFile<R: Read + Seek> {
    header: AsepriteFileHeader,
    parser: Parser<R>,
    cur_frame: usize,
    layers: Vec<Layer>,
    frames: Vec<Frame>,
}

impl<R: Read + Seek> AsepriteFile<R> {
    fn parse(r: R) -> Result<Self, AsepriteError> {
        let mut parser = Parser::new(r);

        let header = AsepriteFileHeader::parse(&mut parser)?;
        assert_eq!(header.magic, constants::ASE_FILE_MAGIC);

        parser.seek(128)?;

        let mut file = AsepriteFile {
            header,
            parser,
            cur_frame: 0,
            layers: Vec::new(),
            frames: Vec::new(),
        };

        while let Some(frame) = file.next_frame()? {
            file.frames.push(frame);
        }

        Ok(file)
    }

    fn next_frame(&mut self) -> Result<Option<Frame>, AsepriteError> {
        if self.cur_frame >= self.header.frames as usize {
            return Ok(None);
        }
        self.cur_frame += 1;
        let size: u32 = self.parser.next()?;
        let magic: u16 = self.parser.next()?;
        let chunks: u16 = self.parser.next()?;
        let duration: u16 = self.parser.next()?;
        self.parser.skip(6)?;
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
            self.apply_chunk(&mut frame)?;
        }

        for (i, l) in self.layers.iter().enumerate() {
            if l.visible() {
                frame.image.draw(0, 0, &frame.layers[i], l.opacity);
            }
        }

        Ok(Some(frame))
    }

    fn apply_chunk(&mut self, frame: &mut Frame) -> Result<(), AsepriteError> {
        let chunk_pos = self.parser.position();
        let chunk_size: u32 = self.parser.next()?;
        let chunk_type: u16 = self.parser.next()?;
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
            constants::ASE_FILE_CHUNK_CEL => {
                let layer_index: u16 = self.parser.next()?;
                let x: i16 = self.parser.next()?;
                let y: i16 = self.parser.next()?;
                let opacity: u8 = self.parser.next()?;
                let cel_type: u16 = self.parser.next()?;
                self.parser.skip(7)?;

                match cel_type {
                    constants::ASE_FILE_COMPRESSED_CEL => {
                        let w: u16 = self.parser.next()?;
                        let h: u16 = self.parser.next()?;
                        let data = self.parser.next_n(chunk_end - self.parser.position())?;
                        // For some reason inflate uses a String instead of an Error.
                        let data = inflate::inflate_bytes_zlib(data)
                            .map_err(AsepriteError::CorruptFile)?;
                        let cel = Image::new_from_data(w, h, data);
                        frame.layers[layer_index as usize].draw(x, y, &cel, opacity);
                    }
                    constants::ASE_FILE_LINK_CEL => {
                        let linked_frame: u16 = self.parser.next()?;
                        let cel = &self.frames[linked_frame as usize].layers[layer_index as usize];
                        frame.layers[layer_index as usize].draw(x, y, cel, opacity);
                    }
                    ct => {
                        return Err(AsepriteError::Unimplemented(format!(
                            "unhandled cel type 0x{:x}",
                            ct
                        )));
                    }
                }
            }
            constants::ASE_FILE_CHUNK_LAYER => {
                let flags = self.parser.next()?;
                let layer_type = self.parser.next()?;
                let child_level = self.parser.next()?;
                let default_width = self.parser.next()?;
                let default_height = self.parser.next()?;
                let blend_mode = self.parser.next()?;
                let opacity = self.parser.next()?;
                self.parser.skip(3)?;
                let name = self.parser.next()?;

                self.layers.push(Layer {
                    flags,
                    layer_type,
                    child_level,
                    default_width,
                    default_height,
                    blend_mode,
                    opacity,
                    name,
                })
            }
            ct => {
                return Err(AsepriteError::Unimplemented(format!(
                    "unhandled chunk type 0x{:x}",
                    ct
                )));
            }
        }

        self.parser.advance_to(chunk_end)?;

        Ok(())
    }
}

#[derive(Debug)]
enum AsepriteError {
    Unimplemented(String),
    CorruptFile(String),
    Error(Box<dyn Error>),
}

impl Display for AsepriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        f.write_str("error")?;
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

        let ase = AsepriteFile::parse(f)?;

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
