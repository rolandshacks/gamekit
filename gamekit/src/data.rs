//!
//! Data
//!

use crate::{api::{Disposable, LockRef}, error::Error, manifest::StaticDataDescriptor};

pub struct StaticData {
    data: &'static [u8]
}

pub type StaticDataRef = std::sync::Arc<StaticData>;
pub type StaticDataLockRef = LockRef<StaticData>;

impl Disposable for StaticData {
    fn dispose(&mut self) {
    }
}

impl StaticData {
    pub fn from_resource(descriptor: &StaticDataDescriptor) -> Result<Self, Error> {
        Self::from_memory(descriptor.data)
    }

    pub fn from_memory(data: &'static [u8]) -> Result<Self, Error> {
        Ok(Self { data })
    }

    pub fn data(&self) -> &'static [u8] {
        self.data
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}
