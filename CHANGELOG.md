# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

[Unreleased]: https://github.com/nik-rev/auto-default/compare/v0.2.1...HEAD

## [v0.2.1] - 2026-01-14

[v0.2.1]: https://github.com/nik-rev/auto-default/compare/v0.2.0...v0.2.1

## [v0.2.0] - 2026-01-14

[v0.2.0]: https://github.com/nik-rev/auto-default/compare/v0.1.5...v0.2.0

### Removed

It is now an error to use `#[auto_default(skip)]` on a field that has a default value:

```rust
#[auto_default]
struct User {
    #[auto_default(skip)]
    age: u32 = 4,
}
```

The `#[auto_default(skip)]` attribute will do nothing.

## [v0.1.5] - 2026-01-14

[v0.1.5]: https://github.com/nik-rev/auto-default/compare/v0.1.4...v0.1.5

## [v0.1.4] - 2026-01-14

[v0.1.4]: https://github.com/nik-rev/auto-default/compare/v0.1.3...v0.1.4

## [v0.1.3] - 2026-01-14

[v0.1.3]: https://github.com/nik-rev/auto-default/compare/v0.1.2...v0.1.3

## [v0.1.2] - 2026-01-14

[v0.1.2]: https://github.com/nik-rev/auto-default/compare/v0.1.1...v0.1.2

## [v0.1.1] - 2026-01-13

[v0.1.1]: https://github.com/nik-rev/auto-default/compare/v0.1.0...v0.1.1

## [v0.1.0] - 2026-01-13

[v0.1.0]: https://github.com/nik-rev/auto-default/releases/v0.1.0
