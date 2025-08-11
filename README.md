# ResourceCache

## A CLI utility for efficient web resource caching built with Rust.

## Quickstart

```sh
# Clone
git clone https://github.com/Adel-Ayoub/ResourceCache.git
cd ResourceCache

# Build + install
./build.sh && ./install.sh

# First run (shows help)
resource-cache --help
```

---

## Command palette

```text
CLI utility to retrieve resources from the web with a cache

Usage: resource-cache [OPTIONS] <COMMAND>

Commands:
  get            
  stats          
  clean-expired  remove expired files
  delete         delete all the files in the cache
  help           Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  location of the config file, default: $HOME/.config/resource-cache/config.conf
  -h, --help             Print help
  -V, --version          Print version
```

### get
```text
Usage: resource-cache get [OPTIONS] <URL>

Arguments:
  <URL>

Options:
  -o, --output <FILENAME>
  -r, --refresh
  -e, --expire-time <EXPIRE_TIME>  time offset in seconds when the file will expire
  -h, --help                       Print help
```

---

## Architecture overview

```text
stdin ─┐                                           ┌─ stdout/stderr
       ▼                                           ▼
+------------------+       args/help        +-----------------------+
|       CLI        | ─────────────────────▶ | Orchestrator (main.rs)|
|  resource-cache  | ◀──────────────────────|  parses + dispatches  |
+------------------+                        +-----------------------+
                 │                                      │
                 │                                      ▼
                 │                         +-------------------------+
                 │                         | Config loader (config.rs)|
                 │                         | expands $VARS, builds    |
                 │                         | absolute paths, ensures  |
                 │                         | parent directories       |
                 │                         +-------------------------+
                 │                                      │
                 ▼                                      ▼
        +------------------+          HTTP              +--------------------+
        |  Cacher (lib.rs) | ========================> | Reqwest on Tokio   |
        |   cache policy   | <======================== | download bytes      |
        +------------------+                           +--------------------+
                 │
                 ▼
        +-----------------------------------------------+
        | Filesystem (cache-dir)                        |
        | - write file                                  |
        | - update cache.json metadata                  |
        +-----------------------------------------------+
```
----

## Usage Examples

```sh
# Download and cache
resource-cache get -o wallpaper.png "https://images.alphacoders.com/135/thumb-1920-1352190.png"

# Force re-download (ignore cache)
resource-cache get -o data.json --refresh "https://httpbin.org/json"

# Custom expiration (1 day)
resource-cache get -o doc.pdf --expire-time 86400 "https://example.com/file.pdf"

# Inspect cache
resource-cache stats

# Remove expired files only
resource-cache clean-expired

# Remove everything in the cache
resource-cache delete
```

---

## Configuration

The tool automatically creates a configuration file at `$HOME/.config/resource-cache/config.conf`:

```ini
cache-db = "$HOME/.cache/resource-cache/cache.json"
cache-dir = "$HOME/.cache/resource-cache/cache/"
random-offset-range = "-36000..36001"
file-default-lifetime = 1728000
```

### Configuration Options
- **cache-db**: JSON file storing cache metadata
- **cache-dir**: Directory for storing cached files
- **random-offset-range**: Random expiration offset in seconds
- **file-default-lifetime**: Default file lifetime in seconds


---

## On‑disk layout

```text
$HOME/.cache/resource-cache/
├── cache/                 # downloaded files
└── cache.json             # metadata (URL → path, times, expire)
```

---

## Install

```sh
# Build artifacts and completion files
./build.sh

# Install binary and completions
./install.sh
```

Manual installation (alternative):
```sh
cargo build --release --out-dir ./target -Z unstable-options
install -Dm755 ./target/resource-cache /usr/local/bin/resource-cache
```

---

## Notes

- Re-running `get` with the same URL uses the cached file unless `--refresh` is set.
- When the output filename already exists in the cache, numeric prefixes are added to avoid collision.
- Expiration is computed from `file-default-lifetime` plus a random offset within `random-offset-range`, unless `--expire-time` is specified for that file.

---

## Development

```sh
# Dev build
cargo build

# Release build with explicit artifact directory
cargo build --release --out-dir ./target -Z unstable-options

# Tests and checks
cargo test
cargo clippy
```

## License

MIT License - see [LICENSE](LICENSE) file for details.


