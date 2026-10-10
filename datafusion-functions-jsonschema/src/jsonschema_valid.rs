//! The `jsonschema_valid` scalar UDF. It answers only whether an instance conforms, so validation stops at the
//! first error.

use std::sync::{
    Arc,
    LazyLock,
    OnceLock,
};

use datafusion::{
    arrow::{
        array::BooleanBuilder,
        datatypes::{
            DataType,
            Field,
            FieldRef,
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

use crate::{
    jiter_json,
    schema_argument::SchemaArgument,
    string_argument::StringArgument,
};

static DOCUMENTATION: LazyLock<Documentation> = LazyLock::new(|| {
    Documentation::builder(
        DocSection {
            include: true,
            label: "JSON Schema Functions",
            description: None,
        },
        "Returns true if the instance conforms to the schema and false if it does not. \
         Returns null if either argument is null or the instance is not JSON. \
         A schema that is not a valid JSON Schema, has an unresolvable $ref, or names an unsupported draft \
         is an error; a literal schema fails at planning. The draft comes from $schema, else 2020-12. \
         No $ref is retrieved from the network or the filesystem.",
        "jsonschema_valid(instance, schema)",
    )
    .with_argument("instance", "JSON text to validate, as Utf8, LargeUtf8 or Utf8View.")
    .with_argument("schema", "JSON Schema text, as Utf8, LargeUtf8 or Utf8View; a literal or a column.")
    .with_sql_example(
        "SELECT id FROM tool_calls \
         WHERE NOT jsonschema_valid(arguments, '{\"type\": \"object\", \"required\": [\"amount\"]}');",
    )
    .build()
});

/// Whether a JSON instance conforms to a JSON Schema.
#[must_use]
pub fn jsonschema_valid(instance: Expr, schema: Expr) -> Expr {
    jsonschema_valid_udf().call(vec![instance, schema])
}

/// The shared `jsonschema_valid` UDF.
#[must_use]
pub fn jsonschema_valid_udf() -> Arc<ScalarUDF> {
    static UDF: OnceLock<Arc<ScalarUDF>> = OnceLock::new();
    let udf = UDF.get_or_init(|| Arc::new(ScalarUDF::from(JsonschemaValid::new())));
    Arc::clone(udf)
}

/// Implements `jsonschema_valid` for `DataFusion`.
#[derive(Debug, PartialEq, Eq, Hash)]
struct JsonschemaValid {
    signature: Signature,
}

impl JsonschemaValid {
    fn new() -> Self {
        let string = Coercion::new_exact(TypeSignatureClass::Native(logical_string()));
        Self {
            signature: Signature::coercible(vec![string.clone(), string], Volatility::Immutable),
        }
    }
}

impl ScalarUDFImpl for JsonschemaValid {
    fn name(&self) -> &'static str {
        "jsonschema_valid"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Ok(DataType::Boolean)
    }

    fn return_field_from_args(&self, args: ReturnFieldArgs) -> Result<FieldRef> {
        let schema_literal = args.scalar_arguments.get(1).copied().flatten();
        SchemaArgument::check_literal(self.name(), schema_literal)?;
        Ok(Arc::new(Field::new(self.name(), DataType::Boolean, true)))
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [instance, schema] = args.args.as_slice() else {
            return exec_err!("jsonschema_valid expects 2 arguments");
        };
        let instances = StringArgument::try_from(instance)?;
        let schemas = SchemaArgument::try_from(schema)?;

        let mut results = BooleanBuilder::with_capacity(args.number_rows);
        for row in 0..args.number_rows {
            let Some(schema) = schemas.schema(self.name(), row)? else {
                results.append_null();
                continue;
            };
            let Some(text) = instances.value(row) else {
                results.append_null();
                continue;
            };
            // An instance that is not JSON cannot be assessed, so it is null rather than false.
            let Ok(instance) = jiter_json::parse(text) else {
                results.append_null();
                continue;
            };
            results.append_value(schema.validator().is_valid(&instance));
        }
        Ok(ColumnarValue::Array(Arc::new(results.finish())))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(&DOCUMENTATION)
    }
}
