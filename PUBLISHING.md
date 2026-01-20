# Publishing Hope OS

This guide explains how to publish Hope OS to both crates.io (Rust) and PyPI (Python).

## Prerequisites

### For crates.io

1. Create account at https://crates.io
2. Get API token from https://crates.io/settings/tokens
3. Login:
```bash
cargo login <your-token>
```

### For PyPI

1. Create account at https://pypi.org
2. Get API token from https://pypi.org/manage/account/token/
3. Configure in `~/.pypirc`:
```ini
[pypi]
username = __token__
password = pypi-<your-token>
```

4. Install maturin:
```bash
pip install maturin
```

## Publishing to crates.io

### Quick Method

```bash
# Linux/macOS
./scripts/publish-crates.sh

# Windows
scripts\publish-crates.bat
```

### Manual Method

```bash
# 1. Run tests
cargo test --release

# 2. Check clippy
cargo clippy --all-targets

# 3. Format check
cargo fmt -- --check

# 4. Dry run
cargo publish --dry-run

# 5. Publish
cargo publish
```

### Result

- URL: https://crates.io/crates/hope-os
- Install: `cargo add hope-os`

## Publishing to PyPI

### Quick Method

```bash
# Linux/macOS
./scripts/publish-pypi.sh

# Windows
scripts\publish-pypi.bat
```

### Manual Method

```bash
# 1. Build wheels
maturin build --release --features python

# 2. Install locally for testing
pip install target/wheels/hope_os-*.whl

# 3. Run tests
pytest tests/

# 4. Publish
maturin publish --features python
```

### Result

- URL: https://pypi.org/project/hope-os/
- Install: `pip install hope-os`

## Version Management

Update version in:
1. `Cargo.toml` - `version = "x.y.z"`
2. `pyproject.toml` - `version = "x.y.z"`
3. `CHANGELOG.md` - Add new version section

```bash
# Bump version
sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml
sed -i 's/version = "0.1.0"/version = "0.2.0"/' pyproject.toml
```

## Platform Support

### crates.io
- All platforms Rust supports
- No pre-built binaries (compiled on install)

### PyPI (via maturin)
- Windows (x64)
- macOS (x64, ARM64)
- Linux (x64, ARM64)
- Pre-built wheels for Python 3.8-3.12

## CI/CD Publishing

Add to GitHub Actions (`.github/workflows/publish.yml`):

```yaml
name: Publish

on:
  release:
    types: [published]

jobs:
  publish-crates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}

  publish-pypi:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          command: publish
          args: --features python
        env:
          MATURIN_PYPI_TOKEN: ${{ secrets.PYPI_TOKEN }}
```

## Checklist Before Publishing

- [ ] All tests pass
- [ ] Version bumped
- [ ] CHANGELOG updated
- [ ] README is up to date
- [ ] No sensitive data in code
- [ ] License file present
- [ ] Documentation builds

---

()=>[]
