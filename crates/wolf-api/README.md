# Wolf API

`wolf-api` provides the hand-written [`WolfApi`](src/lib.rs) client facade and
data types generated from Wolf's vendored OpenAPI 3.1 schema.

## Updating the Wolf schema

From a machine with a running Wolf instance:

```sh
curl --unix-socket /var/run/wolf/wolf.sock \
  http://localhost/api/v1/openapi-schema \
  > crates/wolf-api/openapi/wolf.openapi.json
cargo run -p wolf-api-gen
```

Review the resulting diff in [`src/types.rs`](src/types.rs). That diff is the
API change. Do not edit the generated file directly; the `wolf-api-gen`
freshness test verifies it against the vendored schema in CI.
