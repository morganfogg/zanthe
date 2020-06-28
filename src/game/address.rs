//! The locations of important information in the
//! header section of the story file.

// Common to all versions
pub const VERSION: usize = 0x0;
pub const FLAGS_1: usize = 0x1;
pub const HIGH_MEMORY_BASE: usize = 0x4;
pub const PROGRAM_COUNTER_STARTS: usize = 0x6;
pub const DICTIONARY_LOCATION: usize = 0x8;
pub const OBJECT_TABLE_LOCATION: usize = 0xA;
pub const GLOBAL_VARIABLE_TABLE_LOCATION: usize = 0xC;
pub const STATIC_MEMORY_BASE: usize = 0xE;
pub const FLAGS_2: usize = 0x10;
pub const _STANDARD_REVISION_NUMBER: usize = 0x32;

pub mod flags1_bits_pre_v4 {
    pub const STATUS_LINE_UNAVAILABLE: u16 = 4;
    pub const SCREEN_SPLITTING_AVAILABLE: u16 = 5;
    pub const VARIABLE_PITCH_FONT_DEFAULT: u16 = 6;
}

pub mod flags1_bits_post_v4 {
    pub const COLOR_AVAILABLE: u16 = 0;
    pub const PICTURE_DISPLAYING_AVAILABLE: u16 = 1;
    pub const BOLD_AVAILABLE: u16 = 2;
    pub const ITALICS_AVAILABLE: u16 = 3;
    pub const FIXED_WIDTH_AVAILABLE: u16 = 4;
    pub const SOUND_EFFECTS_AVAILABLE: u16 = 5;
    pub const TIMED_INPUT_AVAILABLE: u16 = 7;
}

pub mod flags2 {
    pub const TRANSCRIPTING_ON: u16 = 0;
    pub const _FORCE_FIXED_PITCH: u16 = 1;
    pub const _REQUEST_REDRAW: u16 = 2;
    pub const PICTURE_SUPPORT: u16 = 3;
    pub const UNDO_SUPPORT: u16 = 4;
    pub const MOUSE_SUPPORT: u16 = 5;
    pub const COLOR_SUPPORT: u16 = 6;
    pub const SOUND_EFFECT_SUPPORT: u16 = 7;
    pub const MENU_SUPPORT: u16 = 8;
}

// Version 2+
pub const ABBREVIATION_TABLE_LOCATION: usize = 0x18;

// Version 3+
pub const FILE_LENGTH: usize = 0x1A; // Not present in some early version 3 files
pub const CHECKSUM: usize = 0x1C; // ditto

// Version 4+
pub const _INTERPRETER_NUMBER: usize = 0x1E;
pub const _INTERPRETER_VERSION: usize = 0x1F;
pub const _SCREEN_HEIGHT_PRE_Z5: usize = 0x20; // Changed in version 5
pub const _SCREEN_WIDTH_PRE_Z5: usize = 0x21; // ditto

// Version 5+
pub const _SCREEN_WIDTH_POST_Z5: usize = 0x22;
pub const _SCREEN_HEIGHT_POST_Z5: usize = 0x24;
pub const _FONT_WIDTH: usize = 0x26;
pub const _FONT_HEIGHT: usize = 0x27;
pub const _DEFAULT_BACKGROUND_COLOUR: usize = 0x2C;
pub const _DEFAULT_FOREGROUND_COLOR: usize = 0x2D;
pub const _TERMINATING_CHARACTER_TABLE_LOCATION: usize = 0x2E;
pub const ALPHABET_TABLE_LOCATION: usize = 0x34;
pub const HEADER_EXTENSION_TABLE_LOCATION: usize = 0x36;

/// Header extension table value offsets
pub const _EXTENSION_TABLE_REMAINING_WORDS: usize = 0x0;
pub const _MOUSE_CLICK_COORDS_X: usize = 0x1;
pub const _MOUSE_CLICK_COORDS_Y: usize = 0x2;
pub const UNICODE_TRANSLATION_TABLE_LOCATION: usize = 0x3;
pub const _FLAGS_3: usize = 0x4;
pub const _TRUE_DEFAULT_FOREGROUND_COLOR: usize = 0x5;
pub const _TRUE_DEFAULT_BACKGROUND_COLOR: usize = 0x5;
