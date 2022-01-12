#![allow(unused)]

pub const LAYER_VISIBLE: u16 = 1 << 0;
pub const LAYER_EDITABLE: u16 = 1 << 1;
pub const LAYER_LOCKMOVE: u16 = 1 << 2;
pub const LAYER_BACKGROUND: u16 = 1 << 3;
pub const LAYER_CONTINUOUS: u16 = 1 << 4;
pub const LAYER_COLLAPSED: u16 = 1 << 5;
pub const LAYER_REFERENCE: u16 = 1 << 6;

pub const ASE_FILE_MAGIC: u16 = 0xA5E0;
pub const ASE_FILE_FRAME_MAGIC: u16 = 0xF1FA;

pub const ASE_FILE_FLAG_LAYER_WITH_OPACITY: u16 = 1;

pub const ASE_FILE_CHUNK_FLI_COLOR2: u16 = 4;
pub const ASE_FILE_CHUNK_FLI_COLOR: u16 = 11;
pub const ASE_FILE_CHUNK_LAYER: u16 = 0x2004;
pub const ASE_FILE_CHUNK_CEL: u16 = 0x2005;
pub const ASE_FILE_CHUNK_CEL_EXTRA: u16 = 0x2006;
pub const ASE_FILE_CHUNK_COLOR_PROFILE: u16 = 0x2007;
pub const ASE_FILE_CHUNK_MASK: u16 = 0x2016;
pub const ASE_FILE_CHUNK_PATH: u16 = 0x2017;
pub const ASE_FILE_CHUNK_TAGS: u16 = 0x2018;
pub const ASE_FILE_CHUNK_PALETTE: u16 = 0x2019;
pub const ASE_FILE_CHUNK_USER_DATA: u16 = 0x2020;
pub const ASE_FILE_CHUNK_SLICES: u16 = 0x2021;
pub const ASE_FILE_CHUNK_SLICE: u16 = 0x2022;
pub const ASE_FILE_CHUNK_TILESET: u16 = 0x2023;

pub const ASE_FILE_LAYER_IMAGE: u16 = 0;
pub const ASE_FILE_LAYER_GROUP: u16 = 1;

pub const ASE_FILE_RAW_CEL: u16 = 0;
pub const ASE_FILE_LINK_CEL: u16 = 1;
pub const ASE_FILE_COMPRESSED_CEL: u16 = 2;

pub const ASE_FILE_NO_COLOR_PROFILE: u16 = 0;
pub const ASE_FILE_SRGB_COLOR_PROFILE: u16 = 1;
pub const ASE_FILE_ICC_COLOR_PROFILE: u16 = 2;

pub const ASE_COLOR_PROFILE_FLAG_GAMMA: u16 = 1;

pub const ASE_PALETTE_FLAG_HAS_NAME: u16 = 1;

pub const ASE_USER_DATA_FLAG_HAS_TEXT: u32 = 1;
pub const ASE_USER_DATA_FLAG_HAS_COLOR: u32 = 2;

pub const ASE_CEL_EXTRA_FLAG_PRECISE_BOUNDS: u16 = 1;

pub const ASE_SLICE_FLAG_HAS_CENTER_BOUNDS: u32 = 1;
pub const ASE_SLICE_FLAG_HAS_PIVOT_POINT: u32 = 2;
