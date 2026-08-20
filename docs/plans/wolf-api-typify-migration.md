# Plan: replace `openapi-to-rust` with typify-based, checked-in type generation in `wolf-api`

Status: implemented
Scope: `crates/wolf-api` (generator + generated types), small mechanical renames in `crates/wolf-ui`

## Context

`crates/wolf-api` generates its Rust data model from Wolf's OpenAPI 3.1 schema
(`crates/wolf-api/openapi/wolf.openapi.json`, vendored from a running Wolf's
`GET /api/v1/openapi-schema`). Today `build.rs` runs `openapi-to-rust = "0.5"`
at build time, with client/SSE generation disabled — we only consume the
**types**. The HTTP client (`WolfApi` facade, transports, error handling, SSE
parsing) is hand-written and good; it stays.

Problems with the status quo:

- `openapi-to-rust` is niche (~24k total downloads, effectively one org,
  `0.5` → `0.12` in six weeks with breaking changes). We are pinned to an old
  version of an already unpopular tool.
- Generated names are hostile (`RflReflectorWolfCoreEventsLobbyReflType`,
  `WolfApiCreateLobbyRequestRunner`), leaking Wolf's C++ reflection internals
  into `wolf-ui` domain code.
- The generated code lives only in `OUT_DIR`: schema updates produce no
  reviewable diff, and every cold build (including Docker builds) compiles a
  ~25k-line generator as a build dependency.
- The Wolf schema itself is quirky (see below), and the quirks are patched
  ad hoc inside `build.rs`.

Goal: keep `wolf.openapi.json` as the single source of truth, make updating it
to a newer Wolf a cheap, reviewable operation, and stop depending on a fragile
generator.

## Known Wolf schema quirks (all must be handled)

These are confirmed both in our vendored schema and upstream
([games-on-whales/wolf#274](https://github.com/games-on-whales/wolf/issues/274)):

1. `minItems`/`maxItems` are JSON **strings** (`"minItems": "16"`).
2. Nullability is expressed as `anyOf: [X, {"type": "null"}]` (OpenAPI 3.1 /
   JSON Schema 2020-12 style).
3. The app/lobby `runner` union is `anyOf: [AppCMD__tagged, AppDocker__tagged]`
   — really a disjoint tagged union (`type: "process" | "docker"`), repeated
   inline in 4 places.
4. Schema names are mangled C++ reflection names
   (`rfl__Reflector_wolf__core__events__App___ReflType`, `wolf__api__...`,
   `wolf__config__...__tagged`).

The schema is otherwise plain: no `oneOf`, `const`, `prefixItems`,
`discriminator`, or exotic 2020-12 constructs. 28 paths, 42 component schemas.

## Options considered

| Option | Verdict |
|---|---|
| **Update `openapi-to-rust` to 0.12** | Rejected. Keeps us on a high-churn, single-maintainer crate; names stay ugly; no reviewable diffs; solves nothing architectural. |
| **progenitor (Oxide)** | Rejected. OpenAPI 3.0.x only ([progenitor#762](https://github.com/oxidecomputer/progenitor/issues/762), still open); also generates a whole client we don't want — our hand-written `WolfApi` facade is better fitted to Unix sockets + SSE. |
| **openapi-generator (Java CLI)** | Rejected. Heavyweight toolchain dependency, mediocre Rust output, still chokes on the 3.1 quirks. |
| **Hand-written types + schema contract tests** | Rejected. Abandons "schema is the source of truth"; drift becomes a human problem. |
| **typify (Oxide) over `components.schemas` + small normalization pass, generated code checked in** | **Chosen.** |

Why typify fits:

- We only need **types**, and typify is exactly a JSON-Schema→Rust-types
  compiler. OpenAPI 3.1 schemas are JSON Schema; the Wolf subset parses as
  draft-07 (`schemars 0.8` model) once normalized.
- Maintained by Oxide, 27M+ downloads, active releases (0.7.0, June 2026);
  it is the type engine under progenitor.
- Supports `with_derive("PartialEq")` (we currently post-process the generated
  text with string replacement to add PartialEq — that hack dies).
- Known typify weakness (`anyOf` handling) is neutralized by the normalization
  pass converting Wolf's `anyOf` (all disjoint) to `oneOf`.

### Validated by prototype

A throwaway prototype (typify 0.7 + ~60-line normalization) was run against the
vendored schema. Results:

- All 42 schemas map 1:1 to clean names with **no collisions**
  (`App`, `Lobby`, `CreateLobbyRequest`, `ClientSettings`, ...).
- 76 Rust types generated; compile cleanly; serde round-trips verified for:
  `Lobby` with `process` and `docker` runners, nullable `icon_png_path`,
  `StopLobbyEvent` with omitted `pin` (serializes without the key),
  `GenericErrorResponse`.
- `anyOf [X, null]` → `Option<X>` with `#[serde(default, skip_serializing_if)]`.
- The runner union becomes one shared `#[serde(untagged)] enum Runner {
  Cmd(AppCmd), Docker(AppDocker) }` (untagged deserialization is unambiguous
  because each variant carries a single-value `type` enum; serialization still
  emits `"type"` since it is a struct field).

## Target architecture

```
crates/wolf-api/openapi/wolf.openapi.json   # vendored verbatim from Wolf (source of truth)
crates/wolf-api-gen/                        # small bin crate: normalize + typify + write file
crates/wolf-api/src/types.rs                # CHECKED-IN generated code (@generated header)
crates/wolf-api/src/{client,endpoints,...}  # hand-written facade, unchanged in shape
```

- `wolf-api` loses its `build.rs` and **all build-dependencies**. It becomes a
  plain crate: fast cold builds, readable types in-repo, rust-analyzer-friendly.
- Updating Wolf = replace `wolf.openapi.json` → run generator → **review the
  diff of `types.rs`** → fix any compile fallout in the facade. The API change
  is visible in the PR.
- A freshness test in `wolf-api-gen` regenerates in-memory and compares against
  the checked-in file, so CI (`cargo test --workspace`) fails if anyone edits
  `types.rs` by hand or forgets to regenerate after a schema bump.

## Implementation steps

### 1. Create `crates/wolf-api-gen`

Bin crate, workspace member (picked up by the existing `crates/*` glob; it is
not in `default-members`, so `dx serve`/default builds don't touch it).

```toml
[package]
name = "wolf-api-gen"
# workspace version/edition/etc.

[dependencies]
typify = "=0.7.0"        # pin exactly: output must be deterministic
schemars = "0.8"          # for schemars::schema::RootSchema (typify 0.7's input model)
serde_json = "1"
rustfmt-wrapper = "0.2"   # format checked-in output so `cargo fmt --check` passes
```

CI runs `cargo clippy --workspace -- -D warnings` with pedantic/nursery warns
and `unwrap_used`/`expect_used` warns; this is a dev tool, so put
`#![allow(clippy::expect_used, clippy::unwrap_used)]` (or use proper error
returns) at the top of `main.rs` — do not fight the linter here.

### 2. Normalization pass (in `wolf-api-gen`, pure `serde_json::Value` → `Value`)

Input: the vendored spec, byte-identical to Wolf's output. Never edit the
vendored file itself. Steps, in order:

1. Take `components.schemas` (map of name → schema).
2. **Rename** every schema key and fail loudly on collision:
   - strip `rfl__Reflector_` prefix and `___ReflType` suffix if both present;
   - strip a trailing `__tagged`;
   - take the last `__`-separated segment.
   - (`rfl__Reflector_wolf__core__events__App___ReflType` → `App`,
     `wolf__api__AppListResponse` → `AppListResponse`,
     `wolf__config__AppCMD__tagged` → `AppCMD`,
     `wolf__core__virtual_display__DisplayMode` → `DisplayMode`.)
3. Rewrite every `$ref` `#/components/schemas/<old>` →
   `#/definitions/<renamed>`.
4. Coerce string `minItems`/`maxItems` to integers (recursive, same as today's
   `normalize_numeric_constraints`).
5. Convert every `anyOf` → `oneOf` (recursive). Safe because Wolf only emits
   disjoint `anyOf`s; typify then produces `Option` for nullable pairs and
   proper enums for unions.
6. **Hoist the runner union**: replace every occurrence of
   `{"oneOf": [{"$ref": ".../AppCMD"}, {"$ref": ".../AppDocker"}]}` with
   `{"$ref": "#/definitions/Runner"}` and add a `Runner` definition holding
   that union. This collapses 4 duplicate inline enums into one shared type
   used by `App`, `Lobby`, `CreateLobbyRequest`, and `RunnerStartRequest`.
7. Assemble a draft-07 root document:
   `{"$schema": "http://json-schema.org/draft-07/schema#", "title": "WolfApiTypes", "type": "object", "definitions": {...}}`.

If a future schema contains something the pass doesn't recognize (e.g. a
non-disjoint `anyOf` or a construct that fails to parse as `RootSchema`), the
generator must **error out**, not guess.

### 3. Generation (in `wolf-api-gen`)

```rust
let root: schemars::schema::RootSchema = serde_json::from_value(normalized)?;
let mut settings = TypeSpaceSettings::default();
settings.with_derive("PartialEq".to_string());
let mut type_space = TypeSpace::new(&settings);
type_space.add_root_schema(root)?;
let code = rustfmt_wrapper::rustfmt(type_space.to_stream().to_string())?;
```

Write to `crates/wolf-api/src/types.rs` with a header:

```rust
// @generated by wolf-api-gen from openapi/wolf.openapi.json — do not edit.
// Regenerate with: cargo run -p wolf-api-gen
#![allow(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(missing_docs)]
```

(Inner attributes work because `lib.rs` declares it as `pub mod types;`.)

### 4. Freshness test (in `wolf-api-gen`)

A `#[test]` that runs the same normalize+generate pipeline in memory and
asserts equality with the on-disk `crates/wolf-api/src/types.rs`, with a clear
failure message ("types.rs is stale; run `cargo run -p wolf-api-gen`"). The
existing CI `cargo test --workspace` job picks this up automatically — no CI
changes needed.

### 5. Slim down `wolf-api`

- Delete `crates/wolf-api/build.rs`.
- Remove `[build-dependencies]` (`openapi-to-rust`, `prettyplease`,
  `serde_json`, `syn`) from `crates/wolf-api/Cargo.toml`.
- In `lib.rs`, replace the `include!(OUT_DIR)` module with `pub mod types;`.

### 6. Mechanical renames in consumers

`crates/wolf-api` (`client.rs`, `endpoints/*.rs`) and `crates/wolf-ui`
(`domain/app_actions.rs`, `domain/apps.rs`, `domain/settings.rs`). Mapping:

| Old | New |
|---|---|
| `RflReflectorWolfCoreEventsAppReflType` | `App` |
| `RflReflectorWolfCoreEventsLobbyReflType` | `Lobby` |
| `RflReflectorWolfCoreEventsProfileReflType` | `Profile` |
| `RflReflectorWolfCoreEventsStreamSessionReflType` | `StreamSession` |
| `RflReflectorWolfCoreEventsAppReflTypeRunner` | `Runner` |
| `RflReflectorWolfCoreEventsLobbyReflTypeRunner` | `Runner` |
| `WolfApiCreateLobbyRequestRunner` | `Runner` |
| `...Runner::WolfConfigAppCMDTagged(x)` | `Runner::Cmd(x)` |
| `...Runner::WolfConfigAppDockerTagged(x)` | `Runner::Docker(x)` |
| `WolfConfigAppCMDTagged` | `AppCmd` |
| `WolfConfigAppDockerTagged` | `AppDocker` |
| `WolfApiXxx` (requests/responses) | `Xxx` (e.g. `AppListResponse`, `CreateLobbyRequest`, `GenericErrorResponse`, `PartialClientSettings`) |
| `WolfCoreEventsXxx` (events/settings) | `Xxx` (e.g. `JoinLobbyEvent`, `StopLobbyEvent`, `AudioSettings`, `VideoSettings`) |
| `WolfConfigClientSettings` | `ClientSettings` |

Notes:

- The existing `pub type App = types::...` aliases in `endpoints/*.rs` stay —
  they are the stable facade — but now point at readable names.
- `endpoints/events.rs` builds a `Lobby` struct literally in
  `CreateLobbyEvent::into_lobby`; field names are unchanged, only the type
  path changes.
- Field `type` on `AppCmd`/`AppDocker` is generated as `type_` with
  `#[serde(rename = "type")]`; adjust any construction sites.
- Expect small mechanical fallout; the compiler is the checklist. Behavior
  must not change — existing tests in `client.rs`, `endpoints/events.rs`, and
  `wolf-ui` domain tests must pass unmodified except for type-name updates.

### 7. Document the schema-update workflow

Add a short section to `crates/wolf-api` (README or `lib.rs` doc comment):

```sh
# From a machine with a running Wolf instance:
curl --unix-socket /var/run/wolf/wolf.sock \
  http://localhost/api/v1/openapi-schema > crates/wolf-api/openapi/wolf.openapi.json
cargo run -p wolf-api-gen
# Review the diff of crates/wolf-api/src/types.rs — that IS the API change.
```

## Acceptance criteria

- [x] `openapi-to-rust` is gone from `Cargo.lock`.
- [x] `wolf-api` has no `build.rs` and no build-dependencies.
- [x] `crates/wolf-api/src/types.rs` is checked in, `@generated`, rustfmt-clean.
- [x] `cargo run -p wolf-api-gen` is idempotent (second run = no diff).
- [x] Freshness test fails if `types.rs` is edited or the schema changes
      without regeneration.
- [x] `cargo fmt --check`, `cargo clippy --workspace --no-default-features
      --features desktop -- -D warnings`, and `cargo test --workspace
      --no-default-features --features desktop` all pass.
- [x] No behavior change: same JSON on the wire (round-trip tests above), same
      public facade (`WolfApi`, endpoint modules, `WolfEvent`).

## Risks and mitigations

- **typify 0.7 output changes between versions** → pin `=0.7.0`; the freshness
  test makes any accidental drift loud. Version bumps become deliberate PRs
  with a visible `types.rs` diff.
- **Future Wolf schemas use 2020-12 constructs typify 0.7 can't parse**
  (e.g. `prefixItems`) → the generator errors at regeneration time, not at
  runtime; typify's next generation (typify2) targets 2020-12 and is the
  upgrade path.
- **Untagged `Runner` enum misdeserializes** → impossible today: variants are
  discriminated by single-value `type` enums; the round-trip tests cover both
  variants. If Wolf ever adds a third runner, regeneration surfaces it as a
  compile-visible diff.
- **Someone hand-edits `types.rs`** → `@generated` header + freshness test.

## Out of scope (deliberately)

- No generated HTTP client. The hand-written `WolfApi` facade encodes real
  product knowledge (Unix socket transport, SSE parsing, error envelope,
  stream timeouts) that no generator reproduces.
- No `paths`/operation codegen. 28 endpoints are already covered by ~7 small,
  tested, hand-written modules.
- No automatic schema fetching in CI. Updating Wolf is a deliberate, reviewed
  act.
