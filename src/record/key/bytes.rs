use std::sync::Arc;

use arrow::array::{BinaryArray, Datum};

use super::{Key, KeyRef};

impl<'r> KeyRef<'r> for &'r [u8] {
    type Key = bytes::Bytes;

    fn to_key(self) -> Self::Key {
        bytes::Bytes::copy_from_slice(self)
    }
}

impl Key for bytes::Bytes {
    type Ref<'r> = &'r [u8];

    fn as_key_ref(&self) -> Self::Ref<'_> {
        self.as_ref()
    }

    fn to_arrow_datum(&self) -> Arc<dyn Datum> {
        Arc::new(BinaryArray::new_scalar(self.as_ref()))
    }
}
