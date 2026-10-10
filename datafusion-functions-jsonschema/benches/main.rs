//! Benchmarks for the JSON Schema UDFs, invoked directly on Arrow batches so that SQL planning is not measured.

#![expect(
    missing_docs,
    reason = "criterion_group! generates an undocumented public function"
)]

use std::sync::Arc;

use codspeed_criterion_compat::{
    BenchmarkGroup,
    BenchmarkId,
    Criterion,
    Throughput,
    criterion_group,
    criterion_main,
    measurement::WallTime,
};
use datafusion::{
    arrow::{
        array::{
            ArrayRef,
            StringArray,
        },
        datatypes::Field,
    },
    common::ScalarValue,
    config::ConfigOptions,
    logical_expr::{
        ColumnarValue,
        ReturnFieldArgs,
        ScalarFunctionArgs,
        ScalarUDF,
    },
};
use datafusion_functions_jsonschema::udfs::{
    jsonschema_errors_udf,
    jsonschema_valid_udf,
};

/// Rows per batch: the `DataFusion` default batch size of 8,192, and smaller powers of two down to 1,024.
const BATCH_SIZES: [usize; 4] = [1024, 2048, 4096, 8192];

/// One required, typed property.
const SIMPLE_SCHEMA: &str = r#"{
    "type": "object",
    "properties": {
        "id": {"type": "integer"}
    },
    "required": ["id"]
}"#;

/// Typed properties, a numeric and a length bound, an array of strings and a required list.
const MEDIUM_SCHEMA: &str = r#"{
    "type": "object",
    "properties": {
        "id": {"type": "integer", "minimum": 0},
        "name": {"type": "string", "maxLength": 32},
        "tags": {"type": "array", "items": {"type": "string"}}
    },
    "required": ["id", "name"]
}"#;

/// An order: local `$ref`s, a `pattern`, an `enum`, `uniqueItems`, a closed nested object, a `oneOf` and
/// `additionalProperties`.
const COMPLEX_SCHEMA: &str = r##"{
    "$defs": {
        "tag": {"type": "string", "pattern": "^[a-z]+$"},
        "line": {
            "type": "object",
            "properties": {
                "sku": {"type": "string", "minLength": 1},
                "quantity": {"type": "integer", "minimum": 1}
            },
            "required": ["sku", "quantity"],
            "additionalProperties": false
        }
    },
    "type": "object",
    "properties": {
        "id": {"type": "integer", "minimum": 0},
        "name": {"type": "string", "maxLength": 32},
        "status": {"enum": ["new", "paid", "shipped"]},
        "tags": {"type": "array", "items": {"$ref": "#/$defs/tag"}, "uniqueItems": true},
        "lines": {"type": "array", "items": {"$ref": "#/$defs/line"}, "minItems": 1},
        "shipping": {
            "oneOf": [
                {"type": "null"},
                {
                    "type": "object",
                    "properties": {"country": {"type": "string", "minLength": 2, "maxLength": 2}},
                    "required": ["country"]
                }
            ]
        }
    },
    "required": ["id", "name", "status", "lines"],
    "additionalProperties": false
}"##;

fn bench_jsonschema_valid(c: &mut Criterion) {
    let udf = jsonschema_valid_udf();

    bench_case(
        c,
        "jsonschema_valid_literal_valid",
        &udf,
        Instances::Valid,
        Schema::Literal,
    );
    bench_case(
        c,
        "jsonschema_valid_literal_invalid",
        &udf,
        Instances::Invalid,
        Schema::Literal,
    );
    bench_case(
        c,
        "jsonschema_valid_column",
        &udf,
        Instances::Valid,
        Schema::Column,
    );
}

fn bench_jsonschema_errors(c: &mut Criterion) {
    let udf = jsonschema_errors_udf();

    bench_case(
        c,
        "jsonschema_errors_literal_valid",
        &udf,
        Instances::Valid,
        Schema::Literal,
    );
    bench_case(
        c,
        "jsonschema_errors_literal_invalid",
        &udf,
        Instances::Invalid,
        Schema::Literal,
    );
    bench_case(
        c,
        "jsonschema_errors_column",
        &udf,
        Instances::Invalid,
        Schema::Column,
    );
}

criterion_group!(benches, bench_jsonschema_valid, bench_jsonschema_errors);
criterion_main!(benches);

/// Run one case as a group, with one benchmark per schema complexity and size in [`BATCH_SIZES`].
fn bench_case(
    c: &mut Criterion,
    name: &str,
    udf: &ScalarUDF,
    instances: Instances,
    schema: Schema,
) {
    let mut group = c.benchmark_group(name);
    for complexity in Complexity::ALL {
        for rows in BATCH_SIZES {
            bench_udf(
                &mut group,
                BenchmarkId::new(complexity.name(), rows),
                udf,
                instances.array(complexity, rows),
                schema.argument(complexity, rows),
            );
        }
    }
    group.finish();
}

/// Time `udf(instances, schema)` over one batch, as the group's benchmark `id`.
fn bench_udf(
    group: &mut BenchmarkGroup<'_, WallTime>,
    id: BenchmarkId,
    udf: &ScalarUDF,
    instances: ArrayRef,
    schema: ColumnarValue,
) {
    let rows = instances.len();
    let arg_fields = vec![
        Arc::new(Field::new("instance", instances.data_type().clone(), true)),
        Arc::new(Field::new("schema", schema.data_type(), true)),
    ];
    // A literal schema is passed to planning, as the SQL planner would.
    let schema_literal = match &schema {
        ColumnarValue::Scalar(value) => Some(value),
        ColumnarValue::Array(_) => None,
    };
    let return_field = udf
        .return_field_from_args(ReturnFieldArgs {
            arg_fields: &arg_fields,
            scalar_arguments: &[None, schema_literal],
        })
        .expect("the benchmark schema plans");
    let args = vec![ColumnarValue::Array(instances), schema];
    let config_options = Arc::new(ConfigOptions::default());

    // Reporting rows per second makes the batch sizes comparable with each other.
    group.throughput(Throughput::Elements(rows as u64));
    group.bench_function(id, |b| {
        b.iter(|| {
            udf.invoke_with_args(ScalarFunctionArgs {
                args: args.clone(),
                arg_fields: arg_fields.clone(),
                number_rows: rows,
                return_field: Arc::clone(&return_field),
                config_options: Arc::clone(&config_options),
            })
            .expect("the benchmark batch validates without an execution error")
        });
    });
}

/// How much a schema asks of the validator. Each level's instances are shaped for its schema, so a more
/// complex schema also means a larger document per row.
#[derive(Clone, Copy)]
enum Complexity {
    /// [`SIMPLE_SCHEMA`].
    Simple,
    /// [`MEDIUM_SCHEMA`].
    Medium,
    /// [`COMPLEX_SCHEMA`].
    Complex,
}

impl Complexity {
    const ALL: [Self; 3] = [Self::Simple, Self::Medium, Self::Complex];

    fn name(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Medium => "medium",
            Self::Complex => "complex",
        }
    }

    fn schema(self) -> &'static str {
        match self {
            Self::Simple => SIMPLE_SCHEMA,
            Self::Medium => MEDIUM_SCHEMA,
            Self::Complex => COMPLEX_SCHEMA,
        }
    }
}

/// Which instances a benchmark validates.
#[derive(Clone, Copy)]
enum Instances {
    /// Every row conforms to its schema.
    Valid,
    /// Every row breaks its schema: once for the simple schema, three times for the medium one and six times
    /// for the complex one.
    Invalid,
}

impl Instances {
    fn array(self, complexity: Complexity, rows: usize) -> ArrayRef {
        let documents = (0..rows).map(|i| self.document(complexity, i));
        Arc::new(StringArray::from_iter_values(documents))
    }

    /// The instance for row `i`, made distinct by `i` so that no two rows are the same text.
    fn document(self, complexity: Complexity, i: usize) -> String {
        // Starts at 1, because `-0` equals 0 and would pass `minimum`.
        let negative = i + 1;
        match (self, complexity) {
            (Self::Valid, Complexity::Simple) => format!(r#"{{"id":{i}}}"#),
            // A string id.
            (Self::Invalid, Complexity::Simple) => format!(r#"{{"id":"{i}"}}"#),
            (Self::Valid, Complexity::Medium) => {
                format!(r#"{{"id":{i},"name":"item-{i}","tags":["a","b"]}}"#)
            }
            // A negative id, a missing name and a non-string tag.
            (Self::Invalid, Complexity::Medium) => {
                format!(r#"{{"id":-{negative},"tags":["a",{i}]}}"#)
            }
            (Self::Valid, Complexity::Complex) => format!(
                r#"{{"id":{i},"name":"order-{i}","status":"paid","tags":["gift","fragile"],"lines":[{{"sku":"sku-{i}","quantity":1}},{{"sku":"sku-0","quantity":2}}],"shipping":{{"country":"ES"}}}}"#
            ),
            // A negative id, a missing name, an unknown status, a repeated tag, a zero quantity and an unknown
            // property.
            (Self::Invalid, Complexity::Complex) => format!(
                r#"{{"id":-{negative},"status":"lost","tags":["gift","gift"],"lines":[{{"sku":"sku-{i}","quantity":0}}],"note":"{i}"}}"#
            ),
        }
    }
}

/// How a benchmark passes the schema.
#[derive(Clone, Copy)]
enum Schema {
    /// One literal for the whole batch, compiled before the batch runs.
    Literal,
    /// A column holding the same schema text on every row, as a join against a schema table produces.
    Column,
}

impl Schema {
    fn argument(self, complexity: Complexity, rows: usize) -> ColumnarValue {
        let text = complexity.schema();
        match self {
            Self::Literal => ColumnarValue::Scalar(ScalarValue::Utf8(Some(text.to_string()))),
            Self::Column => ColumnarValue::Array(Arc::new(StringArray::from_iter_values(
                std::iter::repeat_n(text, rows),
            ))),
        }
    }
}
