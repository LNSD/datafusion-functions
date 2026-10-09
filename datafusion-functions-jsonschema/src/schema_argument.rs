//! The schema argument of a JSON Schema UDF. A literal schema is compiled once per batch; a schema column is
//! compiled per row through the shared cache.

use datafusion::{
    common::{
        DataFusionError,
        Result,
        ScalarValue,
        exec_err,
        plan_err,
    },
    logical_expr::ColumnarValue,
};

use crate::{
    compiled_schema::CompiledSchema,
    string_argument::StringArgument,
};

/// A schema argument, compiled where it is constant.
pub(crate) enum SchemaArgument {
    /// A literal schema, or `None` for a null literal.
    Constant(Option<CompiledSchema>),
    /// A schema column, compiled per row.
    PerRow(StringArgument),
}

impl SchemaArgument {
    /// Checks a literal schema at planning time, so a bad one fails the query before any data is read. A
    /// schema that is not a literal is accepted here and checked per row.
    ///
    /// # Errors
    ///
    /// Returns a plan error if the literal schema does not compile.
    pub(crate) fn check_literal(function: &str, literal: Option<&ScalarValue>) -> Result<()> {
        let Some(text) = literal.and_then(string_literal) else {
            return Ok(());
        };
        match CompiledSchema::cached(text) {
            Ok(_) => Ok(()),
            Err(err) => plan_err!("{function}: {err}"),
        }
    }

    /// The compiled schema for `row`, or `None` if the schema is null.
    ///
    /// # Errors
    ///
    /// Returns an execution error naming the row if a per-row schema does not compile.
    pub(crate) fn schema(&self, function: &str, row: usize) -> Result<Option<CompiledSchema>> {
        match self {
            Self::Constant(schema) => Ok(schema.clone()),
            Self::PerRow(column) => {
                let Some(text) = column.value(row) else {
                    return Ok(None);
                };
                match CompiledSchema::cached(text) {
                    Ok(schema) => Ok(Some(schema)),
                    Err(err) => exec_err!("{function}: row {row}: {err}"),
                }
            }
        }
    }
}

impl TryFrom<&ColumnarValue> for SchemaArgument {
    type Error = DataFusionError;

    fn try_from(value: &ColumnarValue) -> Result<Self> {
        match value {
            ColumnarValue::Scalar(scalar) => {
                let Some(text) = string_literal(scalar) else {
                    return Ok(Self::Constant(None));
                };
                // Planning already compiled this literal through `check_literal`, but `DataFusion` gives
                // `return_field_from_args` no way to hand the result to execution, so it is fetched again from
                // the cache. The error arm is reached only if the cache evicted it and compilation now fails,
                // which it cannot for the same text.
                match CompiledSchema::cached(text) {
                    Ok(schema) => Ok(Self::Constant(Some(schema))),
                    Err(err) => exec_err!("{err}"),
                }
            }
            ColumnarValue::Array(_) => Ok(Self::PerRow(StringArgument::try_from(value)?)),
        }
    }
}

/// The text of a non-null string literal.
fn string_literal(scalar: &ScalarValue) -> Option<&str> {
    match scalar {
        ScalarValue::Utf8(text) | ScalarValue::LargeUtf8(text) | ScalarValue::Utf8View(text) => {
            text.as_deref()
        }
        _ => None,
    }
}
