//! This module models sf01 record data.

use std::borrow::Borrow;

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::esdata::sf01record::Sf01GenericRecord;

#[derive(Debug, CopyGetters, Getters, Serialize, Deserialize)]
pub struct Sf01GenericRecordSource {
    #[getset(get_copy = "pub")]
    class: u32,
    #[getset(get = "pub")]
    content: String,
}

impl Sf01GenericRecordSource {
    pub fn from_compiled<H: Borrow<Sf01GenericRecord>>(record: H) -> Self {
        let record = record.borrow();

        Self {
            class: record.id(),
            content: base122_rs::encode(record.payload()),
        }
    }
}
