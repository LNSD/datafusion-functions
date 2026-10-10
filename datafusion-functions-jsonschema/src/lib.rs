//! JSON Schema functions for Apache `DataFusion`: scalar UDFs that check JSON text columns against a JSON Schema
//! and report where a document breaks it, without ever reading a `$ref` from the network or the filesystem.

use datafusion::{
    common::Result,
    execution::FunctionRegistry,
};

mod compiled_schema;
mod jsonschema_errors;
mod jsonschema_valid;
mod schema_argument;
mod string_argument;

/// Expression builders for the `DataFusion` `DataFrame` API.
pub mod functions {
    pub use crate::{
        jsonschema_errors::jsonschema_errors,
        jsonschema_valid::jsonschema_valid,
    };
}

/// Cached scalar UDFs, for registering functions individually.
pub mod udfs {
    pub use crate::{
        jsonschema_errors::jsonschema_errors_udf,
        jsonschema_valid::jsonschema_valid_udf,
    };
}

/// Register every JSON Schema UDF with `registry`. No function name or alias collides with a `DataFusion`
/// built-in, so registering replaces none.
///
/// # Errors
///
/// Returns an error if the registry rejects a function.
pub fn register_all(registry: &mut dyn FunctionRegistry) -> Result<()> {
    registry.register_udf(udfs::jsonschema_errors_udf())?;
    registry.register_udf(udfs::jsonschema_valid_udf())?;
    Ok(())
}
