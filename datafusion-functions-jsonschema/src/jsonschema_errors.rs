//! The `jsonschema_errors` scalar UDF. Each error names where the instance failed and which keyword failed it,
//! never the rejected value, so the output cannot leak payload contents.

use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        LazyLock,
        OnceLock,
    },
};

use datafusion::{
    arrow::{
        array::{
            BooleanArray,
            ListArray,
            StringArray,
            StructArray,
        },
        buffer::{
            NullBuffer,
            OffsetBuffer,
        },
        datatypes::{
            DataType,
            Field,
            FieldRef,
            Fields,
        },
    },
    common::{
        Result,
        exec_err,
        types::logical_string,
    },
    logical_expr::{
        Coercion,
        ColumnarValue,
        DocSection,
        Documentation,
        Expr,
        ReturnFieldArgs,
        ScalarFunctionArgs,
        ScalarUDF,
        ScalarUDFImpl,
        Signature,
        TypeSignatureClass,
        Volatility,
    },
};
use serde_json::Value;

use crate::{
    schema_argument::SchemaArgument,
    string_argument::StringArgument,
};

/// The maximum number of errors reported for one instance. Past it, the row's `truncated` field is true.
const MAX_ERRORS: NonZeroUsize = NonZeroUsize::new(100).expect("100 is a nonzero error limit");

static DOCUMENTATION: LazyLock<Documentation> = LazyLock::new(|| {
    Documentation::builder(
        DocSection {
            include: true,
            label: "JSON Schema Functions",
            description: None,
        },
        "Returns a struct of errors and truncated. errors lists each place the instance breaks the schema as \
         instance_path and schema_path JSON Pointers and the failing keyword, and is empty for a conforming \
         instance; rejected values are never included. At most 100 errors are listed, and truncated is true \
         when more exist. Returns null if either argument is null or the instance is not JSON. Schema faults \
         are errors, as in jsonschema_valid.",
        "jsonschema_errors(instance, schema)",
    )
    .with_argument("instance", "JSON text to validate, as Utf8, LargeUtf8 or Utf8View.")
    .with_argument("schema", "JSON Schema text, as Utf8, LargeUtf8 or Utf8View; a literal or a column.")
    .with_sql_example(
        "SELECT id, unnest(jsonschema_errors(arguments, schema)['errors']) \
         FROM tool_calls JOIN tools USING (tool_name);",
    )
    .build()
});

/// Where a JSON instance breaks a JSON Schema, as a struct of the error list and a truncation flag.
#[must_use]
pub fn jsonschema_errors(instance: Expr, schema: Expr) -> Expr {
    jsonschema_errors_udf().call(vec![instance, schema])
}

/// The shared `jsonschema_errors` UDF.
#[must_use]
pub fn jsonschema_errors_udf() -> Arc<ScalarUDF> {
    static UDF: OnceLock<Arc<ScalarUDF>> = OnceLock::new();
    let udf = UDF.get_or_init(|| Arc::new(ScalarUDF::from(JsonschemaErrors::new())));
    Arc::clone(udf)
}

/// Implements `jsonschema_errors` for `DataFusion`.
#[derive(Debug, PartialEq, Eq, Hash)]
struct JsonschemaErrors {
    signature: Signature,
}

impl JsonschemaErrors {
    fn new() -> Self {
        let string = Coercion::new_exact(TypeSignatureClass::Native(logical_string()));
        Self {
            signature: Signature::coercible(vec![string.clone(), string], Volatility::Immutable),
        }
    }
}

impl ScalarUDFImpl for JsonschemaErrors {
    fn name(&self) -> &'static str {
        "jsonschema_errors"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Ok(DataType::Struct(report_fields()))
    }

    fn return_field_from_args(&self, args: ReturnFieldArgs) -> Result<FieldRef> {
        let schema_literal = args.scalar_arguments.get(1).copied().flatten();
        SchemaArgument::check_literal(self.name(), schema_literal)?;
        Ok(Arc::new(Field::new(
            self.name(),
            DataType::Struct(report_fields()),
            true,
        )))
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [instance, schema] = args.args.as_slice() else {
            return exec_err!("jsonschema_errors expects 2 arguments");
        };
        let instances = StringArgument::try_from(instance)?;
        let schemas = SchemaArgument::try_from(schema)?;

        let mut reports = Reports::for_rows(args.number_rows);
        for row in 0..args.number_rows {
            let Some(schema) = schemas.schema(self.name(), row)? else {
                reports.push_null();
                continue;
            };
            let Some(text) = instances.value(row) else {
                reports.push_null();
                continue;
            };
            // An instance that is not JSON cannot be assessed, so it is null rather than an empty error list.
            let Ok(instance) = serde_json::from_str::<Value>(text) else {
                reports.push_null();
                continue;
            };
            // One error past the cap is read only to learn that the list is truncated.
            let mut count = 0;
            for error in schema
                .validator()
                .iter_errors(&instance)
                .take(MAX_ERRORS.get() + 1)
            {
                count += 1;
                if count > MAX_ERRORS.get() {
                    break;
                }
                reports.push_error(
                    error.instance_path().to_string(),
                    error.schema_path().to_string(),
                    error.kind().keyword().to_owned(),
                );
            }
            reports.finish_row(count > MAX_ERRORS.get());
        }
        Ok(ColumnarValue::Array(Arc::new(reports.into_array())))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(&DOCUMENTATION)
    }
}

/// The fields of one error entry.
fn error_fields() -> Fields {
    Fields::from(vec![
        Field::new("instance_path", DataType::Utf8, false),
        Field::new("schema_path", DataType::Utf8, false),
        Field::new("keyword", DataType::Utf8, false),
    ])
}

/// The fields of the returned struct.
fn report_fields() -> Fields {
    Fields::from(vec![
        Field::new("errors", DataType::List(error_list_field()), false),
        Field::new("truncated", DataType::Boolean, false),
    ])
}

fn error_list_field() -> FieldRef {
    Arc::new(Field::new_list_field(
        DataType::Struct(error_fields()),
        false,
    ))
}

/// Accumulates one report per row and assembles them into the returned struct array.
struct Reports {
    instance_paths: Vec<String>,
    schema_paths: Vec<String>,
    keywords: Vec<String>,
    error_counts: Vec<usize>,
    pending_count: usize,
    truncated: Vec<bool>,
    valid_rows: Vec<bool>,
}

impl Reports {
    fn for_rows(rows: usize) -> Self {
        Self {
            instance_paths: Vec::new(),
            schema_paths: Vec::new(),
            keywords: Vec::new(),
            error_counts: Vec::with_capacity(rows),
            pending_count: 0,
            truncated: Vec::with_capacity(rows),
            valid_rows: Vec::with_capacity(rows),
        }
    }

    fn push_error(&mut self, instance_path: String, schema_path: String, keyword: String) {
        self.instance_paths.push(instance_path);
        self.schema_paths.push(schema_path);
        self.keywords.push(keyword);
        self.pending_count += 1;
    }

    fn finish_row(&mut self, truncated: bool) {
        self.error_counts.push(self.pending_count);
        self.pending_count = 0;
        self.truncated.push(truncated);
        self.valid_rows.push(true);
    }

    fn push_null(&mut self) {
        self.error_counts.push(0);
        self.truncated.push(false);
        self.valid_rows.push(false);
    }

    fn into_array(self) -> StructArray {
        let entries = StructArray::new(
            error_fields(),
            vec![
                Arc::new(StringArray::from(self.instance_paths)),
                Arc::new(StringArray::from(self.schema_paths)),
                Arc::new(StringArray::from(self.keywords)),
            ],
            None,
        );
        let errors = ListArray::new(
            error_list_field(),
            OffsetBuffer::from_lengths(self.error_counts),
            Arc::new(entries),
            None,
        );
        StructArray::new(
            report_fields(),
            vec![
                Arc::new(errors),
                Arc::new(BooleanArray::from(self.truncated)),
            ],
            Some(NullBuffer::from(self.valid_rows)),
        )
    }
}
