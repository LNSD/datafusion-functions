//! Behavior of the `jsonschema_errors` UDF through SQL and the `DataFrame` API.

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
        datatypes::Int64Type,
    },
    common::Result,
    prelude::{
        SessionContext,
        col,
        lit,
    },
};
use datafusion_functions_jsonschema::{
    functions,
    register_all,
};

const ORDER_SCHEMA: &str =
    r#"{"type": "object", "required": ["currency"], "properties": {"amount": {"type": "number"}}}"#;

/// One row of `jsonschema_errors` output: `None` for a null row, else the errors and the truncation flag.
type Report = Option<(Vec<(String, String, String)>, bool)>;

#[tokio::test]
async fn sql_lists_paths_and_keywords_of_each_error() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#"{"amount": "secret-value"}"#)])?;

    //* When
    let reports = query_reports(
        &ctx,
        &format!("SELECT jsonschema_errors(payload, '{ORDER_SCHEMA}') FROM payloads"),
    )
    .await?;

    //* Then
    let (mut errors, truncated) = reports[0].clone().expect("a JSON instance is assessed");
    errors.sort();
    assert_eq!(
        errors,
        vec![
            (String::new(), "/required".to_owned(), "required".to_owned()),
            (
                "/amount".to_owned(),
                "/properties/amount/type".to_owned(),
                "type".to_owned()
            ),
        ]
    );
    assert!(!truncated);
    Ok(())
}

#[tokio::test]
async fn sql_output_never_contains_the_rejected_value() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#"{"amount": "secret-value"}"#)])?;

    //* When
    let reports = query_reports(
        &ctx,
        &format!("SELECT jsonschema_errors(payload, '{ORDER_SCHEMA}') FROM payloads"),
    )
    .await?;

    //* Then
    let (errors, _) = reports[0].clone().expect("a JSON instance is assessed");
    for (instance_path, schema_path, keyword) in errors {
        for field in [instance_path, schema_path, keyword] {
            assert!(!field.contains("secret-value"), "{field}");
        }
    }
    Ok(())
}

#[tokio::test]
async fn sql_with_conforming_instance_returns_empty_error_list() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#"{"currency": "EUR", "amount": 1}"#)])?;

    //* When
    let reports = query_reports(
        &ctx,
        &format!("SELECT jsonschema_errors(payload, '{ORDER_SCHEMA}') FROM payloads"),
    )
    .await?;

    //* Then
    assert_eq!(reports, vec![Some((vec![], false))]);
    Ok(())
}

#[tokio::test]
async fn sql_with_null_or_unparseable_instance_returns_null() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![None, Some("{\"currency\":")])?;

    //* When
    let reports = query_reports(
        &ctx,
        &format!("SELECT jsonschema_errors(payload, '{ORDER_SCHEMA}') FROM payloads"),
    )
    .await?;

    //* Then
    assert_eq!(reports, vec![None, None]);
    Ok(())
}

#[tokio::test]
async fn sql_with_more_errors_than_the_cap_truncates_to_100_and_flags_it() -> Result<()> {
    //* Given
    let instance = format!("[{}]", vec!["\"x\""; 150].join(","));
    let ctx = context_with_payloads(vec![Some(&instance)])?;

    //* When
    let reports = query_reports(
        &ctx,
        r#"SELECT jsonschema_errors(payload, '{"items": {"type": "integer"}}') FROM payloads"#,
    )
    .await?;

    //* Then
    let (errors, truncated) = reports[0].clone().expect("a JSON instance is assessed");
    assert_eq!(errors.len(), 100);
    assert!(truncated);
    Ok(())
}

#[tokio::test]
async fn sql_with_invalid_literal_schema_fails_at_planning() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some("{}")])?;

    //* When
    let result = ctx
        .sql(r#"SELECT jsonschema_errors(payload, '{"type": 5}') FROM payloads"#)
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
async fn sql_unnest_of_errors_yields_one_row_per_error() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some(r#"{"amount": "x"}"#)])?;

    //* When
    let batches = ctx
        .sql(&format!(
            "SELECT count(*) FROM (SELECT unnest(jsonschema_errors(payload, '{ORDER_SCHEMA}')['errors']) FROM payloads)"
        ))
        .await?
        .collect()
        .await?;

    //* Then
    let count = batches[0].column(0).as_primitive::<Int64Type>().value(0);
    assert_eq!(count, 2);
    Ok(())
}

#[tokio::test]
async fn dataframe_jsonschema_errors_reports_missing_field() -> Result<()> {
    //* Given
    let ctx = context_with_payloads(vec![Some("{}")])?;

    //* When
    let batches = ctx
        .table("payloads")
        .await?
        .select(vec![functions::jsonschema_errors(
            col("payload"),
            lit(ORDER_SCHEMA),
        )])?
        .collect()
        .await?;

    //* Then
    let reports = report_values(batches[0].column(0));
    assert_eq!(
        reports,
        vec![Some((
            vec![(String::new(), "/required".to_owned(), "required".to_owned())],
            false
        ))]
    );
    Ok(())
}

fn context_with_payloads(payloads: Vec<Option<&str>>) -> Result<SessionContext> {
    let mut ctx = SessionContext::new();
    register_all(&mut ctx)?;
    let payloads: ArrayRef = Arc::new(StringArray::from(payloads));
    let batch = RecordBatch::try_from_iter([("payload", payloads)])?;
    ctx.register_batch("payloads", batch)?;
    Ok(ctx)
}

async fn query_reports(ctx: &SessionContext, sql: &str) -> Result<Vec<Report>> {
    let batches = ctx.sql(sql).await?.collect().await?;
    Ok(batches
        .iter()
        .flat_map(|batch| report_values(batch.column(0)))
        .collect())
}

fn report_values(array: &ArrayRef) -> Vec<Report> {
    let reports = array.as_struct();
    let errors = reports.column(0).as_list::<i32>();
    let truncated = reports.column(1).as_boolean();
    (0..reports.len())
        .map(|row| {
            if reports.is_null(row) {
                return None;
            }
            let entries = errors.value(row);
            let entries = entries.as_struct();
            let instance_paths = entries.column(0).as_string::<i32>();
            let schema_paths = entries.column(1).as_string::<i32>();
            let keywords = entries.column(2).as_string::<i32>();
            let row_errors = (0..entries.len())
                .map(|entry| {
                    (
                        instance_paths.value(entry).to_owned(),
                        schema_paths.value(entry).to_owned(),
                        keywords.value(entry).to_owned(),
                    )
                })
                .collect();
            Some((row_errors, truncated.value(row)))
        })
        .collect()
}
