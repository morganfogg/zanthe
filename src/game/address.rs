//! The locations of important information in the
//! header section of the story file.

// Common to all versions
pub const VERSION: usize = 0x0;
pub const _FLAGS_1: usize = 0x1;
pub const HIGH_MEMORY_BASE: usize = 0x4;
pub const PROGRAM_COUNTER_STARTS: usize = 0x6;
pub const _DICTIONARY_LOCATION: usize = 0x8;
pub const _OBJECT_TABLE_LOCATION: usize = 0xA;
pub const _GLOBAL_VARIABLE_TABLE_LOCATION: usize = 0xC;
pub const STATIC_MEMORY_BASE: usize = 0xE;
pub const _FLAGS_2: usize = 0x10;
pub const _STANDARD_REVISION_NUMBER: usize = 0x32;

// Version 2+
pub const _ABBREVIATION_TABLE_LOCATION: usize = 0x18;

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
pub const _ALPHABET_TABLE_LOCATION: usize = 0x34;
pub const _HEADER_EXTENSION_TABLE_LOCATION: usize = 0x36;
