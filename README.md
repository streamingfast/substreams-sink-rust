## Substreams Sink Rust

This repository show cases a functional base Rust project that consumes a Substreams `.spkg` (local file only).

To run:

```bash
SUBSTREAMS_API_TOKEN="<StreamingFast API Token>" cargo run https://mainnet.eth.streamingfast.io:443 <path/to/substreams.spkg> <module_name>
```

### Details

The presented Rust project contains a `SubstreamsStream` wrapper that handles automatic reconnection in case of error. It is implemented as a Rust `TryStream` which enable consuming the retryable stream easily using standard Rust syntax:

```rust
let stream = SubstreamsStream::new(...);

loop {
   match stream.next().await {
      None => { /* Stream completed, reached end block */ },
      Some(Ok(BlockResponse::New(data))) => { /* Got a BlockScopedData message */ },
      Some(Err(err)) => { /* Fatal error or retry limit reached */ },
   }
}
```

The `main.rs` file accepts three argument the endpoint to reach (in the form `http(s)?://<url>:<port>`), the local file `.spkg` to use for the request and the output module's name to stream from.

#### Incomplete Implementation

For now `cursor` handling is not properly loaded/saved to database, something that would be required on a production system to ensure the stream is resumed at the right location and that a block is never miss.

The `SubstreamStream` while use in other project probably requires some extra hardening to be sure it's 100% correct in all cases that can happen on a Substreams.

### Protobuf Generation

Protobuf generation is done using [buf](https://buf.build/) which can be installed with:

```bash
brew install bufbuild/buf/buf
```

> **Note** See [other installation methods](https://buf.build/docs/installation/) if not on Mac/Linux (`brew` can be installed on Linux).

#### From a pre-built `.spkg`

If you have an `.spkg` that already contains your Protobuf definitions, you can use it directly. It contains Substreams system Protobuf definitions so you can generate everything in one shot:

```bash
buf generate --exclude-path="google" <path/to/substream.spkg>#format=bin
```

> **Note** An `.spkg` contains recursively all Proto definitions, some you may not desire. You can exclude generation of some element via `--exclude-path="google"` flag. You can specify many separated by a comma.

#### Just Substreams

You can generate against published [Substreams Buf Module](https://buf.build/streamingfast/substreams):

```bash
buf generate buf.build/streamingfast/substreams
```

This will include only Substreams system Protobufs to decode packages and perform RPC operations.

#### Deprecated Notice

You will see `WARN Plugin "buf.build/prost/crate" is deprecated` when generating the code, this is because `buf.build/community/neoeinstein-create:<version>` is not yet available.