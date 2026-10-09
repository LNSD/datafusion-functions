//! Behavior of the `jsonschema_valid` UDF through SQL, the `DataFrame` API, and direct invocation.

use std::sync::Arc;

use datafusion::{
    arrow::{
        array::{
            Array as _,
            ArrayRef,
            AsArray as _,
            RecordBatch,
            StringArray,
        },
        datatypes::{
            DataType,
            Field,
        },
    },
    common::{
        Result,
        ScalarValue,
    },
    config::ConfigOptions,
    logical_expr::{
        ColumnarValue,
        ScalarFunctionArgs,
    },
    prelude::{
        SessionContext,
        col,
        lit,
    },
};
use datafusion_functions_jsonschema::{
    functions,
    register_all,
    udfs::jsonschema_valid_udf,
};

const AMOUNT_SCHEMA: &str =
    r#"{"type": "object", "required": ["amount"], "properties": {"amount": {"type": "number"}}}"#;

#[tokio::test]
async fn sql_with_literal_schema_reports_true_for_conforming_and_false_for_breaking_instances()
-> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![
        Some(r#"{"amount": 12.5}"#),
        Some(r#"{"amount": "12.5"}"#),
        Some("{}"),
    ])?;

    //* When
    let values = query_booleans(
        &ctx,
        &format!("SELECT jsonschema_valid(payload, '{AMOUNT_SCHEMA}') FROM payloads"),
    )
    .await?;

    //* Then
    assert_eq!(values, vec![Some(true), Some(false), Some(false)]);
    Ok(())
}

#[tokio::test]
async fn sql_with_null_instance_or_null_schema_returns_null() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![None, Some("{}")])?;

    //* When
    let null_instance = query_booleans(
        &ctx,
        &format!("SELECT jsonschema_valid(payload, '{AMOUNT_SCHEMA}') FROM payloads"),
    )
    .await?;
    let null_schema =
        query_booleans(&ctx, "SELECT jsonschema_valid(payload, NULL) FROM payloads").await?;

    //* Then
    assert_eq!(null_instance, vec![None, Some(false)]);
    assert_eq!(null_schema, vec![None, None]);
    Ok(())
}

#[tokio::test]
async fn sql_with_instance_that_is_not_json_returns_null_not_false() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#"{"amount": 1"#), Some("not json")])?;

    //* When
    let values = query_booleans(
        &ctx,
        &format!("SELECT jsonschema_valid(payload, '{AMOUNT_SCHEMA}') FROM payloads"),
    )
    .await?;

    //* Then
    assert_eq!(values, vec![None, None]);
    Ok(())
}

#[tokio::test]
async fn sql_with_literal_schema_that_is_not_json_fails_at_planning() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some("{}")])?;

    //* When
    let result = ctx
        .sql("SELECT jsonschema_valid(payload, '{\"type\":') FROM payloads")
        .await;

    //* Then
    let error = result.expect_err("a literal schema that is not JSON must fail planning");
    assert!(error.to_string().contains("schema is not JSON"), "{error}");
    Ok(())
}

#[tokio::test]
async fn sql_with_literal_schema_that_breaks_its_meta_schema_fails_at_planning() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some("{}")])?;

    //* When
    let result = ctx
        .sql(r#"SELECT jsonschema_valid(payload, '{"type": 5}') FROM payloads"#)
        .await;

    //* Then
    let error = result.expect_err("an invalid literal schema must fail planning");
    assert!(
        error.to_string().contains("not a valid JSON Schema"),
        "{error}"
    );
    Ok(())
}

#[tokio::test]
async fn sql_with_literal_schema_with_dangling_local_ref_fails_at_planning() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some("{}")])?;

    //* When
    let result = ctx
        .sql(r##"SELECT jsonschema_valid(payload, '{"$ref": "#/$defs/missing"}') FROM payloads"##)
        .await;

    //* Then
    let error = result.expect_err("a dangling $ref must fail planning");
    assert!(error.to_string().contains("unresolvable $ref"), "{error}");
    Ok(())
}

#[tokio::test]
async fn sql_with_literal_schema_with_remote_ref_fails_without_fetching() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some("{}")])?;

    //* When
    let result = ctx
        .sql(r#"SELECT jsonschema_valid(payload, '{"$ref": "https://example.com/schemas/order.json"}') FROM payloads"#)
        .await;

    //* Then
    let error = result.expect_err("a remote $ref must not resolve");
    let message = error.to_string();
    assert!(message.contains("unresolvable $ref"), "{message}");
    assert!(message.contains("is not retrieved"), "{message}");
    Ok(())
}

#[tokio::test]
async fn sql_with_literal_schema_naming_unknown_draft_fails_at_planning() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some("{}")])?;

    //* When
    let result = ctx
        .sql(r#"SELECT jsonschema_valid(payload, '{"$schema": "https://example.com/draft/99", "type": "object"}') FROM payloads"#)
        .await;

    //* Then
    let error = result.expect_err("an unknown draft must fail planning");
    assert!(error.to_string().contains("unsupported draft"), "{error}");
    Ok(())
}

#[tokio::test]
async fn sql_with_local_ref_into_defs_validates_through_the_reference() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#"{"price": 3}"#), Some(r#"{"price": -3}"#)])?;
    let schema = r##"{"properties": {"price": {"$ref": "#/$defs/positive"}}, "$defs": {"positive": {"type": "number", "minimum": 0}}}"##;

    //* When
    let values = query_booleans(
        &ctx,
        &format!("SELECT jsonschema_valid(payload, '{schema}') FROM payloads"),
    )
    .await?;

    //* Then
    assert_eq!(values, vec![Some(true), Some(false)]);
    Ok(())
}

#[tokio::test]
async fn sql_without_schema_keyword_applies_draft_2020_12() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#"[1, "a"]"#), Some(r#"["a", 1]"#)])?;
    // `prefixItems` exists only from draft 2020-12 on.
    let schema = r#"{"prefixItems": [{"type": "integer"}, {"type": "string"}]}"#;

    //* When
    let values = query_booleans(
        &ctx,
        &format!("SELECT jsonschema_valid(payload, '{schema}') FROM payloads"),
    )
    .await?;

    //* Then
    assert_eq!(values, vec![Some(true), Some(false)]);
    Ok(())
}

#[tokio::test]
async fn sql_with_draft_4_schema_applies_draft_4_keywords() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some("4"), Some("5")])?;
    // Draft 4 spells an exclusive bound as a boolean beside `maximum`.
    let schema = r#"{"$schema": "http://json-schema.org/draft-04/schema#", "maximum": 5, "exclusiveMaximum": true}"#;

    //* When
    let values = query_booleans(
        &ctx,
        &format!("SELECT jsonschema_valid(payload, '{schema}') FROM payloads"),
    )
    .await?;

    //* Then
    assert_eq!(values, vec![Some(true), Some(false)]);
    Ok(())
}

#[tokio::test]
async fn sql_with_draft_2020_12_treats_format_as_annotation() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#""not an email""#)])?;
    let schema = r#"{"type": "string", "format": "email"}"#;

    //* When
    let values = query_booleans(
        &ctx,
        &format!("SELECT jsonschema_valid(payload, '{schema}') FROM payloads"),
    )
    .await?;

    //* Then
    assert_eq!(values, vec![Some(true)]);
    Ok(())
}

#[tokio::test]
async fn sql_with_schema_column_validates_each_row_against_its_own_schema() -> Result<()> {
    //* Given
    let mut ctx = SessionContext::new();
    register_all(&mut ctx)?;
    let batch = RecordBatch::try_from_iter([
        ("payload", strings(vec![Some("1"), Some("1"), Some("1")])),
        (
            "schema",
            strings(vec![
                Some(r#"{"type": "integer"}"#),
                Some(r#"{"type": "string"}"#),
                None,
            ]),
        ),
    ])?;
    ctx.register_batch("calls", batch)?;

    //* When
    let values =
        query_booleans(&ctx, "SELECT jsonschema_valid(payload, schema) FROM calls").await?;

    //* Then
    assert_eq!(values, vec![Some(true), Some(false), None]);
    Ok(())
}

#[tokio::test]
async fn sql_with_invalid_schema_in_column_fails_execution_naming_the_row() -> Result<()> {
    //* Given
    let mut ctx = SessionContext::new();
    register_all(&mut ctx)?;
    let batch = RecordBatch::try_from_iter([
        ("payload", strings(vec![Some("1"), Some("1")])),
        (
            "schema",
            strings(vec![Some(r#"{"type": "integer"}"#), Some(r#"{"type": 5}"#)]),
        ),
    ])?;
    ctx.register_batch("calls", batch)?;

    //* When
    let result = query_booleans(&ctx, "SELECT jsonschema_valid(payload, schema) FROM calls").await;

    //* Then
    let error = result.expect_err("a bad per-row schema must fail execution");
    assert!(error.to_string().contains("row 1"), "{error}");
    Ok(())
}

#[tokio::test]
async fn sql_with_utf8_view_and_large_utf8_arguments_validates_without_cast_errors() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#"{"amount": 1}"#), Some("{}")])?;

    //* When
    let values = query_booleans(
        &ctx,
        &format!(
            "SELECT jsonschema_valid(arrow_cast(payload, 'Utf8View'), arrow_cast('{AMOUNT_SCHEMA}', 'LargeUtf8')) \
             FROM payloads"
        ),
    )
    .await?;

    //* Then
    assert_eq!(values, vec![Some(true), Some(false)]);
    Ok(())
}

#[tokio::test]
async fn dataframe_jsonschema_valid_filters_breaking_instances() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#"{"amount": 1}"#), Some("{}")])?;

    //* When
    let batches = ctx
        .table("payloads")
        .await?
        .select(vec![functions::jsonschema_valid(
            col("payload"),
            lit(AMOUNT_SCHEMA),
        )])?
        .collect()
        .await?;

    //* Then
    assert_eq!(
        boolean_values(batches[0].column(0)),
        vec![Some(true), Some(false)]
    );
    Ok(())
}

#[test]
fn invoke_with_sliced_instances_reads_only_the_slice() -> Result<()> {
    //* Given
    let instances = strings(vec![Some("{}"), Some(r#"{"amount": 1}"#), Some("{}")]).slice(1, 2);

    //* When
    let values = invoke(instances, 2)?;

    //* Then
    assert_eq!(values, vec![Some(true), Some(false)]);
    Ok(())
}

#[test]
fn invoke_with_empty_batch_returns_empty_array() -> Result<()> {
    //* Given
    let instances = strings(vec![]);

    //* When
    let values = invoke(instances, 0)?;

    //* Then
    assert_eq!(values, vec![]);
    Ok(())
}

fn context_with_payloads(payloads: Vec<Option<&str>>) -> Result<SessionContext> {
    let mut ctx = SessionContext::new();
    register_all(&mut ctx)?;
    let batch = RecordBatch::try_from_iter([("payload", strings(payloads))])?;
    ctx.register_batch("payloads", batch)?;
    Ok(ctx)
}

async fn query_booleans(ctx: &SessionContext, sql: &str) -> Result<Vec<Option<bool>>> {
    let batches = ctx.sql(sql).await?.collect().await?;
    Ok(batches
        .iter()
        .flat_map(|batch| boolean_values(batch.column(0)))
        .collect())
}

fn invoke(instances: ArrayRef, number_rows: usize) -> Result<Vec<Option<bool>>> {
    let schema = ColumnarValue::Scalar(ScalarValue::Utf8(Some(AMOUNT_SCHEMA.to_owned())));
    let result = jsonschema_valid_udf().invoke_with_args(ScalarFunctionArgs {
        args: vec![ColumnarValue::Array(instances), schema],
        arg_fields: vec![
            Arc::new(Field::new("instance", DataType::Utf8, true)),
            Arc::new(Field::new("schema", DataType::Utf8, true)),
        ],
        number_rows,
        return_field: Arc::new(Field::new("result", DataType::Boolean, true)),
        config_options: Arc::new(ConfigOptions::default()),
    })?;
    let ColumnarValue::Array(array) = result else {
        panic!("expected an array result");
    };
    Ok(boolean_values(&array))
}

fn strings(values: Vec<Option<&str>>) -> ArrayRef {
    Arc::new(StringArray::from(values))
}

fn boolean_values(array: &ArrayRef) -> Vec<Option<bool>> {
    let booleans = array.as_boolean();
    (0..booleans.len())
        .map(|row| booleans.is_valid(row).then(|| booleans.value(row)))
        .collect()
}
