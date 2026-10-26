---
name: "adr-007-no-network-filesystem-ref"
description: "Resolve schema references only locally and disallow network and filesystem fetching. Load when modifying reference resolution, schema security boundaries, or external resource access"
type: "adr"
status: "accepted"
---

# No Network or Filesystem Access in Schema Reference Resolution

## Context

The JSON Schema specification defines `$ref` pointers to allow schema reuse. In general JSON Schema implementations, `$ref` values may contain URIs referencing remote HTTP/HTTPS servers or relative filesystem paths (`file://`).

In a distributed analytical query engine like DataFusion, allowing queries to trigger arbitrary network requests or read arbitrary host filesystem paths introduces severe security and operational risks:
- Denial of service: slow or hung HTTP requests can block query execution threads indefinitely.
- Server-Side Request Forgery (SSRF): malicious queries could probe internal private cloud infrastructure via schema `$ref` URIs.
- Information disclosure: schemas could read local files from execution nodes.

## Decision

1. **Local reference resolution only.** Only intra-document JSON Pointer references (such as `#/$defs/item` or `#/definitions/entry`) are resolved by the schema compiler (answers [FR-011](prd-001-jsonschema-validation.md#functional-requirements)).
2. **Disallow external fetching.** The schema compiler configures no network retriever and no filesystem loader. Any external `$ref` (HTTP, HTTPS, FTP, file path) is treated as an unresolvable reference (answers [FR-011](prd-001-jsonschema-validation.md#functional-requirements), [SC-003](prd-001-jsonschema-validation.md#success-criteria)).
3. **Compilation failure on external refs.** Any schema containing an external `$ref` fails schema compilation (at query planning for literal schemas, and at execution for column schemas).

## Consequences

- Queries remain strictly isolated and cannot perform network requests or open filesystem handles.
- Hostile or user-generated schemas cannot be exploited for SSRF or local file inclusion.
- Schemas requiring multi-file composition must be bundled into a single document before being passed to `jsonschema_valid` or `jsonschema_errors`.

## Checklist

- [ ] Local references within the same document resolve normally
- [ ] Network retrieval is disabled in the schema compiler
- [ ] Filesystem access is disabled in the schema compiler
- [ ] External references cause schema compilation failure

## References

- [prd-001](prd-001-jsonschema-validation.md) - Source: Product requirements for JSON Schema validation
