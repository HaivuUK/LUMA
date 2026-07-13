---
date: '2026-06-12T15:54:46+01:00'
draft: true
title: 'Building From Source'
category: 'Development'
weight: 402
---

### Building

If you would like to build LUMA your self from source please ensure you have rust installed on your computer.
You can find more information on how to install rust [here](https://rust-lang.org/).

LUMA was last built with 1.96+.

#### Building Distributables

Distributables are build using [tauri build](https://tauri.app/distribute/).

```bash
cargo tauri build
```

#### Debug build

```bash
cargo build
```

#### Release build  

```bash
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test
```

#### Run integration tests

```bash
cargo test --test functional
```

#### Run with debug output

```bash
LUMA_DEBUG=1 cargo test
```
