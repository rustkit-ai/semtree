# semtree-cli

The `semtree` command-line tool, part of the [semtree](https://github.com/rustkit-ai/semtree) workspace.

Parse a codebase into structured chunks, embed them locally, and search by meaning. Everything runs on-device, with no API key or daemon.

## Install

```bash
cargo install semtree-cli
```

## Commands

```bash
semtree init                                   # create .semtree.toml
semtree index ./my-project                     # index (incremental by default)
semtree index ./my-project --full              # force a full re-scan
semtree search "error handling strategy" -k 5  # semantic / hybrid search
semtree context "authentication flow"          # RAG context block for LLMs
semtree stats                                  # chunks, languages, index size
semtree analyze                                # complexity metrics, largest functions
```

All commands accept `--config <path>` to point to a custom `.semtree.toml`.

## Configuration

`semtree init` writes a `.semtree.toml`:

```toml
[embed]
backend = "fastembed"   # fastembed | openai | ollama

[store]
backend = "usearch"     # usearch | qdrant

index_dir = ".semtree"
```

See the [workspace README](https://github.com/rustkit-ai/semtree) for the full backend and language matrix.

## Incremental indexing

Re-running `semtree index` only processes files whose content changed, tracked by a `manifest.json` next to the index. Pass `--full` to force a complete re-scan.

## License

MIT

Part of [rustkit-ai](https://github.com/rustkit-ai) - open source Rust tools for the AI development era.
