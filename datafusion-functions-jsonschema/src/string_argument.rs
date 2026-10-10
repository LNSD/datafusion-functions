//! One string argument of a UDF, downcast once per batch. `Utf8`, `LargeUtf8` and `Utf8View` are read as
//! they are, without a cast to a common type.

use datafusion::{
    arrow::{
        array::{
            Array as _,
            AsArray as _,
            LargeStringArray,
            StringArray,
            StringViewArray,
        },
        datatypes::DataType,
    },
    common::{
        DataFusionError,
        Result,
        exec_err,
    },
    logical_expr::ColumnarValue,
};

/// A string column, or a scalar string that every row reads.
pub(crate) struct StringArgument {
    strings: Strings,
    is_scalar: bool,
}

impl StringArgument {
    /// The string in `row`, or `None` if it is null.
    pub(crate) fn value(&self, row: usize) -> Option<&str> {
        let row = if self.is_scalar { 0 } else { row };
        match &self.strings {
            Strings::Utf8(array) => array.is_valid(row).then(|| array.value(row)),
            Strings::LargeUtf8(array) => array.is_valid(row).then(|| array.value(row)),
            Strings::Utf8View(array) => array.is_valid(row).then(|| array.value(row)),
        }
    }
}

impl TryFrom<&ColumnarValue> for StringArgument {
    type Error = DataFusionError;

    fn try_from(value: &ColumnarValue) -> Result<Self> {
        let (array, is_scalar) = match value {
            ColumnarValue::Array(array) => (array.clone(), false),
            ColumnarValue::Scalar(scalar) => (scalar.to_array_of_size(1)?, true),
        };
        // The signature admits only these three types, so the last arm guards a planner bug, not user input.
        let strings = match array.data_type() {
            DataType::Utf8 => Strings::Utf8(array.as_string::<i32>().clone()),
            DataType::LargeUtf8 => Strings::LargeUtf8(array.as_string::<i64>().clone()),
            DataType::Utf8View => Strings::Utf8View(array.as_string_view().clone()),
            other => return exec_err!("expected a string argument, got {other}"),
        };
        Ok(Self { strings, is_scalar })
    }
}

enum Strings {
    Utf8(StringArray),
    LargeUtf8(LargeStringArray),
    Utf8View(StringViewArray),
}
