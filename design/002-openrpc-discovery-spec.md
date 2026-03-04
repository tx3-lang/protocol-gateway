# OpenRPC Discovery Spec — Tx3 Protocol API Layer

## Overview

This document specifies the addition of runtime API documentation via the
[OpenRPC 1.4.1](https://spec.open-rpc.org/) standard. The server will dynamically generate an
OpenRPC document from the loaded TII registry, allowing clients and developers to discover all
available JSON-RPC methods, their parameters, and response schemas without reading source code or
protocol files.

The documentation is auto-generated — adding a new `.tii` file to `protocols/` and restarting
the server is sufficient for the new methods to appear in the spec.

---

## Motivation

The current server exposes JSON-RPC methods dynamically from `.tii` files, but there is no
runtime mechanism for clients to discover which methods are available, what parameters they
accept, or what responses they return. Developers must inspect raw `.tii` files or read the
source code to understand the API surface.

OpenRPC is the JSON-RPC equivalent of OpenAPI/Swagger. It provides a machine-readable
specification that enables:

- Interactive API exploration via the [OpenRPC Playground](https://playground.open-rpc.org/)
- Client code generation
- Automated testing and validation
- Self-documenting APIs

---

## Transport

### New Endpoints

| Property       | Value                            | Description                                       |
|----------------|----------------------------------|---------------------------------------------------|
| Method         | POST `/`                         | JSON-RPC `rpc.discover` method (standard OpenRPC) |
| Method         | GET `/openrpc.json`              | Direct HTTP access to the OpenRPC document         |
| Method         | GET `/docs`                      | Redirect to interactive OpenRPC Playground         |

The existing `POST /` endpoint for protocol methods remains unchanged.

---

## Methods

### `rpc.discover`

Standard OpenRPC discovery method. Returns the full OpenRPC specification document describing
all available methods.

#### Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "rpc.discover"
}
```

No `params` required.

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "openrpc": "1.4.1",
    "info": {
      "title": "Tx3 Protocol API",
      "version": "0.1.0",
      "description": "JSON-RPC 2.0 API dynamically generated from loaded Tx3 protocols. Active network: mainnet."
    },
    "methods": [ ... ]
  }
}
```

---

### `GET /openrpc.json`

Returns the same OpenRPC document as `rpc.discover`, but via a plain HTTP GET request. This
endpoint is useful for:

- Tools that expect a URL to fetch the spec from (e.g. OpenRPC Playground `schemaUrl`)
- CI/CD pipelines that validate the API surface
- Static documentation generation

**Response:** `200 OK` with `Content-Type: application/json`.

---

### `GET /docs`

Redirects (HTTP 307) to the hosted OpenRPC Playground with a `schemaUrl` query parameter
pointing back to the server's `/openrpc.json` endpoint. This provides an interactive
documentation UI without bundling any frontend code.

**Response:** `307 Temporary Redirect` to:
```
https://playground.open-rpc.org/?schemaUrl=http://localhost:{port}/openrpc.json
```

---

## OpenRPC Document Structure

The generated document follows the [OpenRPC Specification 1.4.1](https://spec.open-rpc.org/).

### Top-Level Fields

```json
{
  "openrpc": "1.4.1",
  "info": {
    "title": "Tx3 Protocol API",
    "version": "0.1.0",
    "description": "JSON-RPC 2.0 API dynamically generated from loaded Tx3 protocols. Active network: {network}."
  },
  "methods": [ ... ]
}
```

### Method Object

For each `<protocol>.<tx>` combination discovered in the registry:

```json
{
  "name": "ticketing-2026.buy_ticket",
  "summary": "Build buy_ticket transaction (ticketing-2026)",
  "description": "Constructs an unsigned Cardano transaction for buy_ticket in the ticketing-2026 protocol.\n\nActive network: mainnet\nProfile pre-fills: issuer, treasury, issuer_beacon_policy, ...\nCaller must supply: buyer",
  "paramStructure": "by-name",
  "params": [
    {
      "name": "buyer",
      "description": "Address",
      "required": true,
      "schema": { "type": "string", "description": "Cardano address (bech32)" }
    },
    {
      "name": "issuer",
      "description": "Address (pre-filled by mainnet profile)",
      "required": false,
      "schema": { "type": "string", "description": "Cardano address (bech32)" }
    }
  ],
  "result": {
    "name": "TxEnvelope",
    "schema": {
      "type": "object",
      "properties": {
        "tx": { "type": "string", "description": "Hex-encoded unsigned CBOR transaction" },
        "hash": { "type": "string", "description": "Transaction hash (optional, if provided by TRP)" }
      },
      "required": ["tx"]
    }
  },
  "errors": [
    { "code": -32000, "message": "Protocol not found" },
    { "code": -32001, "message": "Transaction not found" },
    { "code": -32002, "message": "Arguments mismatch" },
    { "code": -32003, "message": "Build error" },
    { "code": -32004, "message": "Network not found" }
  ]
}
```

### Parameter Classification

Parameters are classified based on the active network profile:

| Category | `required` | Source | Description |
|----------|-----------|--------|-------------|
| Caller-supplied | `true` | Not in profile | Caller **must** provide these in `params` |
| Profile-filled | `false` | Pre-filled by profile | Caller **may** override these, but they have defaults |

This classification is computed by calling `protocol.invoke(tx, Some(network))` and comparing
`params()` (all parameters) against `unspecified_params()` (only those without profile values).

### ParamType to JSON Schema Mapping

The `tx3_sdk::tii::ParamType` enum maps to JSON Schema as follows:

| `ParamType` variant | JSON Schema |
|---------------------|-------------|
| `Bytes` | `{ "type": "string", "description": "Hex-encoded bytes" }` |
| `Integer` | `{ "type": "integer" }` |
| `Boolean` | `{ "type": "boolean" }` |
| `Address` | `{ "type": "string", "description": "Cardano address (bech32)" }` |
| `UtxoRef` | `{ "type": "string", "description": "UTxO reference (txhash#index)" }` |
| `List(inner)` | `{ "type": "array", "items": <inner schema> }` |
| `Custom(schema)` | Serialized `schemars::Schema` (pass-through) |

---

## Example Exchange

### Discovery via JSON-RPC

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "rpc.discover"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "openrpc": "1.4.1",
    "info": {
      "title": "Tx3 Protocol API",
      "version": "0.1.0",
      "description": "JSON-RPC 2.0 API dynamically generated from loaded Tx3 protocols. Active network: mainnet."
    },
    "methods": [
      {
        "name": "ticketing-2026.buy_ticket",
        "summary": "Build buy_ticket transaction (ticketing-2026)",
        "paramStructure": "by-name",
        "params": [
          {
            "name": "buyer",
            "description": "Address",
            "required": true,
            "schema": { "type": "string", "description": "Cardano address (bech32)" }
          }
        ],
        "result": {
          "name": "TxEnvelope",
          "schema": {
            "type": "object",
            "properties": {
              "tx": { "type": "string" },
              "hash": { "type": "string" }
            },
            "required": ["tx"]
          }
        }
      },
      {
        "name": "rpc.discover",
        "summary": "Returns the OpenRPC specification for this API",
        "params": [],
        "result": {
          "name": "OpenRPC Document",
          "schema": { "type": "object" }
        }
      }
    ]
  }
}
```

### Discovery via HTTP GET

```
GET /openrpc.json HTTP/1.1
Host: localhost:8080
```

Returns the same JSON document as the `rpc.discover` result above.

### Interactive Documentation

```
GET /docs HTTP/1.1
Host: localhost:8080
```

Returns `307 Temporary Redirect` to:
```
Location: https://playground.open-rpc.org/?schemaUrl=http://localhost:8080/openrpc.json
```

---

## Module Structure

```
src/
├── main.rs
├── server.rs            # + GET /openrpc.json and GET /docs routes, CORS layer
├── config.rs
├── rpc/
│   ├── mod.rs           # + pub mod discover
│   ├── dispatcher.rs    # + rpc.discover method handling
│   ├── discover.rs      # NEW: OpenRPC generation, HTTP handlers
│   ├── handler.rs
│   └── error.rs
└── registry/
    └── mod.rs           # + iter() method on TiiRegistry
```

### New File: `src/rpc/discover.rs`

Responsibilities:
- `generate_openrpc(state) -> Value` — builds the full OpenRPC document
- `openrpc_handler(State)` — Axum handler for `GET /openrpc.json`
- `docs_redirect()` — Axum handler for `GET /docs`
- `param_type_to_schema(ParamType) -> Value` — maps tx3 types to JSON Schema

### Modified: `src/registry/mod.rs`

Add an `iter()` method to `TiiRegistry` to enumerate all loaded protocols:

```rust
pub fn iter(&self) -> impl Iterator<Item = (&String, &Protocol)> {
    self.protocols.iter()
}
```

### Modified: `src/rpc/dispatcher.rs`

Add `rpc.discover` handling before the `parse_method()` call:

```rust
if request.method == "rpc.discover" {
    let doc = discover::generate_openrpc(&state);
    let response = json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "result": doc,
    });
    return (StatusCode::OK, Json(response));
}
```

### Modified: `src/server.rs`

Add routes and CORS layer:

```rust
use axum::routing::{get, post};
use tower_http::cors::{CorsLayer, Any};

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(dispatcher::dispatch))
        .route("/openrpc.json", get(discover::openrpc_handler))
        .route("/docs", get(discover::docs_redirect))
        .with_state(state)
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
}
```

---

## Dependencies

### New

| Crate | Version | Purpose |
|-------|---------|---------|
| `tower-http` | `0.6` (feature: `cors`) | CORS middleware for OpenRPC Playground cross-origin requests |

### Unchanged

All existing dependencies remain as-is. No OpenRPC-specific library is needed — the document
is built using `serde_json::json!()` macros.

---

## CORS

The OpenRPC Playground is hosted at `playground.open-rpc.org` and fetches the spec via
`/openrpc.json`. Without CORS headers, the browser blocks this cross-origin request.

The CORS layer is applied globally with permissive settings (`Allow-Origin: *`). For
production deployments, this should be tightened to allow only the playground origin.

---

## Generation Flow

```
1. Iterate all protocols in TiiRegistry via iter()
2. For each protocol:
   a. Get transaction map via protocol.txs()
   b. For each transaction:
      i.   Call protocol.invoke(tx_name, Some(network)) to get Invocation with profile
      ii.  Call invocation.params() to get all parameter names and types
      iii. Call invocation.unspecified_params() to identify caller-required params
      iv.  Map each ParamType to JSON Schema
      v.   Build OpenRPC Method object with params classified as required/optional
3. Append the rpc.discover self-documenting method
4. Assemble the complete OpenRPC document
```

---

## Out of Scope

- Custom `BASE_URL` configuration for the playground redirect (hardcodes `localhost:{port}`)
- Caching the generated OpenRPC document (generated on each request; negligible cost)
- OpenRPC `components` / `$ref` usage (inline schemas only)
- Per-request network parameter (network is server-global, as established in 001)
