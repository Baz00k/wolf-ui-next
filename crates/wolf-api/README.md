# Wolf API

`wolf-api` provides the hand-written [`WolfApi`](src/lib.rs) client facade and
data types generated from the Wolf OpenAPI schema vendored by `wolf-api-gen`.

## Updating the Wolf schema

With the development Wolf:

```sh
mise run update-schema
```

The task downloads the schema and regenerates `src/types.rs`. Review and adapt
callers to API changes, then run `mise run check`.

Review the resulting diff in [`src/types.rs`](src/types.rs). That diff is the
API change. Do not edit the generated file directly; the `wolf-api-gen`
freshness test verifies it against the generator's vendored schema in CI.
