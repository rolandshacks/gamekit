//!
//! Built-In
//!

use crate::{self as gamekit, api::Error, material::Materials, resources::Resources};

crate::load!();

pub struct BuiltIns {}

impl BuiltIns {
    pub fn build_resources(stage: usize) -> Result<(), Error> {
        let descriptors = DESCRIPTOR_TABLE;
        Resources::build(descriptors, stage)?;
        Ok(())
    }

    pub fn build_materials() -> Result<(), Error> {
        let descriptors = DESCRIPTOR_TABLE;
        Materials::build(descriptors.materials)?;
        Ok(())
    }
}
