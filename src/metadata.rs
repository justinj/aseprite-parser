use std::io::{Read, Seek};

use crate::{
    constants,
    parser::{Parse, Parser, Skip},
    AsepriteError,
};

/// The header for the entire Aseprite file.
#[derive(Debug)]
pub struct FileHeader {
    pub size: u32,
    pub magic: u16,
    pub frames: u16,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub flags: u32,
    pub speed: u16,
    pub next: u32,
    pub frit: u32,
    pub transparent_index: u32,
    _skip: Skip<3>,
    pub ncolors: u16,
    pub pixel_width: u8,
    pub pixel_height: u8,
    pub grid_x: i16,
    pub grid_y: i16,
    pub grid_width: u16,
    pub grid_height: u16,
}

impl Parse for FileHeader {
    fn parse<R>(p: &mut Parser<R>) -> Result<Self, AsepriteError>
    where
        R: Read + Seek,
    {
        Ok(FileHeader {
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
            _skip: p.next()?,
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
pub struct LayerHeader {
    pub flags: u16,
    pub layer_type: u16,
    pub child_level: u16,
    pub default_width: u16,
    pub default_height: u16,
    pub blend_mode: u16,
    pub opacity: u8,
    _skip: Skip<3>,
    pub name: String,
}

impl LayerHeader {
    pub(crate) fn visible(&self) -> bool {
        self.flags & constants::LAYER_VISIBLE != 0
    }
}

impl Parse for LayerHeader {
    fn parse<R: Read + Seek>(p: &mut Parser<R>) -> Result<Self, AsepriteError> {
        Ok(LayerHeader {
            flags: p.next()?,
            layer_type: p.next()?,
            child_level: p.next()?,
            default_width: p.next()?,
            default_height: p.next()?,
            blend_mode: p.next()?,
            opacity: p.next()?,
            _skip: p.next()?,
            name: p.next()?,
        })
    }
}

#[derive(Debug)]
pub struct Tag {
    pub from: u16,
    pub to: u16,
    pub anidir: u8,
    _skip0: Skip<8>,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    _skip1: Skip<1>,
    pub name: String,
}

impl Parse for Tag {
    fn parse<R: Read + Seek>(p: &mut Parser<R>) -> Result<Self, AsepriteError> {
        Ok(Tag {
            from: p.next()?,
            to: p.next()?,
            anidir: p.next()?,
            _skip0: p.next()?,
            r: p.next()?,
            g: p.next()?,
            b: p.next()?,
            _skip1: p.next()?,
            name: p.next()?,
        })
    }
}

/// A keyframe for a [Slice].
#[derive(Debug)]
pub struct SliceKey {
    pub frame: u32,
    pub bounds: Rect,
    pub center: Option<Rect>,
    pub pivot: Option<Point>,
}

#[derive(Debug)]
pub struct Slice {
    pub name: String,
    pub keys: Vec<SliceKey>,
    pub user_data: UserData,
}

#[derive(Debug, Default)]
pub struct UserData {
    pub string: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Parse for Rect {
    fn parse<R>(p: &mut Parser<R>) -> Result<Self, AsepriteError>
    where
        R: Read + Seek,
    {
        Ok(Rect {
            x: p.next()?,
            y: p.next()?,
            w: p.next()?,
            h: p.next()?,
        })
    }
}

#[derive(Debug, Default)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Parse for Point {
    fn parse<R>(p: &mut Parser<R>) -> Result<Self, AsepriteError>
    where
        R: Read + Seek,
    {
        Ok(Point {
            x: p.next()?,
            y: p.next()?,
        })
    }
}
