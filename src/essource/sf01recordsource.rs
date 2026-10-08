//! This module models sf01 record data.

use std::borrow::Borrow;

use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::{esdata::sf01record::Sf01GenericRecord};

#[derive(Debug, Getters, Serialize, Deserialize)]
pub struct Sf01GenericRecordSource {
    #[getset(get = "pub")]
    class: String,
    #[getset(get = "pub")]
    content: String,
}

impl Sf01GenericRecordSource {
    pub fn from_compiled<H: Borrow<Sf01GenericRecord>>(record: H) -> Self {
        let record = record.borrow();

        Self {
            class: String::from_utf8(record.id().to_vec()).unwrap_or_else(|_| {
                log::warn!(
                    "Record ID {:?} is not a valid string, using integer representation instead.",
                    record.id()
                );
                u32::from_le_bytes(record.id()).to_string()
            }),
            content: base122_rs::encode(record.payload()),
        }
    }
}
