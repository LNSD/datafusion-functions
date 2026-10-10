//! Compiling JSON Schema text into a validator, and caching validators by schema text. Compilation never
//! reads a `$ref` from outside the schema document: no network access or file read happens during a scan.

use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        LazyLock,
        Mutex,
        PoisonError,
    },
};

use jsonschema::{
    PatternOptions,
    ReferencingError,
    Retrieve,
    Uri,
    Validator,
    error::ValidationErrorKind,
};
use lru::LruCache;
use serde_json::Value;

use crate::jiter_json::Jiter;

/// The number of distinct schemas whose validators stay compiled. A query with a per-row schema column
/// recompiles a schema only once it has been evicted, least recently used first.
const CACHE_CAPACITY: NonZeroUsize =
    NonZeroUsize::new(64).expect("64 is a nonzero schema cache capacity");

static CACHE: LazyLock<Mutex<LruCache<String, CompiledSchema>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(CACHE_CAPACITY)));

/// A JSON Schema compiled into a validator. Cloning shares the validator.
#[derive(Clone)]
pub(crate) struct CompiledSchema(Arc<Validator<Jiter>>);

impl CompiledSchema {
    /// The compiled form of `text`, from the shared cache, compiling and caching it on a miss.
    ///
    /// # Errors
    ///
    /// Returns a [`SchemaError`] if `text` does not compile.
    pub(crate) fn cached(text: &str) -> Result<Self, SchemaError> {
        if let Some(schema) = lock_cache().get(text) {
            return Ok(schema.clone());
        }
        // Compile outside the lock, so a slow schema never blocks lookups of others. Two threads may compile
        // the same text at once; both results are equal, and the second insert replaces the first.
        let schema: Self = text.parse()?;
        lock_cache().put(text.to_owned(), schema.clone());
        Ok(schema)
    }

    pub(crate) fn validator(&self) -> &Validator<Jiter> {
        &self.0
    }
}

impl std::str::FromStr for CompiledSchema {
    type Err = SchemaError;

    fn from_str(text: &str) -> Result<Self, SchemaError> {
        let schema: Value =
            serde_json::from_str(text).map_err(|err| SchemaError::NotJson(err.to_string()))?;
        let validator = jsonschema::options_for::<Jiter>()
            .with_retriever(OfflineRetriever)
            // The linear-time engine rules out catastrophic backtracking on a hostile `pattern`, at the cost
            // of rejecting lookaround and backreferences.
            .with_pattern_options(PatternOptions::regex())
            .build(&schema)
            .map_err(|err| match err.kind() {
                ValidationErrorKind::Referencing(ReferencingError::UnknownSpecification {
                    specification,
                }) => SchemaError::UnsupportedDraft(specification.clone()),
                ValidationErrorKind::Referencing(referencing) => {
                    SchemaError::UnresolvableRef(referencing.to_string())
                }
                _ => SchemaError::Invalid(err.to_string()),
            })?;
        Ok(Self(Arc::new(validator)))
    }
}

/// Why schema text could not be compiled. Each variant carries the underlying message, which may quote the
/// schema but never an instance.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SchemaError {
    /// The text is not JSON.
    #[error("schema is not JSON: {0}")]
    NotJson(String),
    /// The JSON is not a valid schema for its draft.
    #[error("schema is not a valid JSON Schema: {0}")]
    Invalid(String),
    /// A `$ref` points outside the schema document, or to a location in it that does not exist.
    #[error("schema has an unresolvable $ref: {0}")]
    UnresolvableRef(String),
    /// `$schema` names a draft the validator does not implement.
    #[error("schema uses an unsupported draft: {0}")]
    UnsupportedDraft(String),
}

/// Refuses every external `$ref`, so resolution stays inside the schema document.
struct OfflineRetriever;

impl Retrieve for OfflineRetriever {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Err(format!("external reference {uri} is not retrieved").into())
    }
}

fn lock_cache() -> std::sync::MutexGuard<'static, LruCache<String, CompiledSchema>> {
    // The cache holds only finished validators, so a panic in another thread cannot leave it half-updated.
    CACHE.lock().unwrap_or_else(PoisonError::into_inner)
}
