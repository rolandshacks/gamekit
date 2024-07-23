//!
//! Constants
//!

/// Constants
pub struct Constants {}

impl Constants {
    pub const ENABLE_VALIDATION_LAYER: bool = false; // default setting, can be overwritten in manifest
    pub const ENABLE_API_DUMP_LAYER: bool = false; // default setting, can be overwritten in manifest
    pub const FRAME_BUFFER_COUNT: usize = 2;
    pub const REQUIRE_EXTENDED_DYNAMIC_STATE: bool = true;   // mandatory feature extension
    pub const REQUIRE_EXTENDED_DYNAMIC_STATE3: bool = false; // optional feature extension
    pub const DEFAULT_BLITTER_BATCH_CAPACITY: usize = 2048;
    pub const DEFAULT_FPS: u32 = 60;
}
