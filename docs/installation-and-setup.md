---
title: Installation
weight: 3
---

Requires Rust 1.89 or newer. Install Rust using [rustup](https://rustup.rs/).

Once the crate is published, add it to your application:

```sh
cargo add spatie-calendar-links
```

Before publication, use a local path dependency:

```toml
[dependencies]
spatie-calendar-links = { path = "../calendar-links-rust" }
```
