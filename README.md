# Tx3 Protocol API Layer

JSON-RPC 2.0 server that dynamically exposes [Tx3](https://github.com/tx3-lang/tx3) protocols. Drop a `.tii` file into `protocols/`, restart the server, and the protocol's transactions become available as RPC methods under a protocol-specific endpoint.

## Quick start

```bash
cargo run
```

The server listens on `http://0.0.0.0:8080` by default.

## Configuration

All configuration is read from environment variables at startup.

| Variable        | Default        | Description                                              |
|-----------------|----------------|----------------------------------------------------------|
| `PORT`          | `8080`         | Port the API server listens on.                          |
| `NETWORK`       | `mainnet`      | Network / profile name (`mainnet`, `preview`, `preprod`).|
| `PROTOCOLS_DIR` | `./protocols`  | Directory scanned for `*.tii` files at startup.          |
| `TRP_URL`       | *(not set)*    | Override the TRP endpoint (uses Demeter public endpoints by default). |
| `TRP_HEADERS`   | *(not set)*    | HTTP headers for the TRP endpoint, comma-separated `key=value` pairs (e.g. `dmtr-api-key=mykey`). Used when `TRP_URL` is set. |

## Routing

Each protocol gets its own namespace under `/{protocol}`:

| Endpoint | Description |
|----------|-------------|
| `GET /` | List all available protocols |
| `POST /{protocol}` | JSON-RPC 2.0 endpoint for a specific protocol |
| `POST /{protocol}` with method `rpc.discover` | Returns the OpenRPC document for the protocol via JSON-RPC |
| `GET /{protocol}/openrpc.json` | Returns the OpenRPC document as plain HTTP |
| `GET /{protocol}/docs` | Redirects to the interactive [OpenRPC Playground](https://playground.open-rpc.org/) |

## API Discovery (OpenRPC)

The server implements [OpenRPC 1.4.1](https://spec.open-rpc.org/) for runtime API discovery. The spec is generated dynamically from the loaded `.tii` files — no manual updates needed.

**List available protocols:**

```bash
curl http://localhost:8080/
```

**Discover available methods for a protocol:**

```bash
curl http://localhost:8080/ticketing-2026/openrpc.json
```

**Open interactive docs:**

```
http://localhost:8080/ticketing-2026/docs
```

## Usage

Each protocol has its own endpoint at `/{protocol}`. Methods are the transaction names directly — no need to prefix with the protocol name.

**Example request:**

```bash
curl -X POST http://localhost:8080/ticketing-2026 \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "buy_ticket",
    "params": {
      "buyer": "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3jcu5d8ps7zex2k2xt3uqxgjqnnj83ws8lhrn648jjxtwq2ytjqp"
    }
  }'
```

**Example response:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tx": "84a500818258204dce7e..."
  }
}
```

## Adding protocols

1. Generate a `.tii` file with [`trix build`](https://github.com/tx3-lang/trix).
2. Copy it into the `protocols/` directory.
3. Restart the server.

No code changes needed — the server discovers protocols dynamically.

## Architecture diagrams

C4 architecture diagrams live in `design/001-assets/` as PlantUML sources. To regenerate the SVGs:

```bash
docker run --rm \
  -v ./design/001-assets:/data \
  plantuml/plantuml -tsvg /data/c4-context.puml /data/c4-container.puml
```

See [design/001-api-scaffolding-spec.md](design/001-api-scaffolding-spec.md) for the scaffolding specification and [design/002-openrpc-discovery-spec.md](design/002-openrpc-discovery-spec.md) for the OpenRPC discovery specification.

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with custom config
NETWORK=preview PORT=3000 cargo run
```
