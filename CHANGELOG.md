# Changelog

## [0.4.0](https://github.com/joshrotenberg/mdbook-lint/compare/v0.3.0...v0.4.0) (2025-08-07)


### Features

* comprehensive corpus testing infrastructure overhaul ([#54](https://github.com/joshrotenberg/mdbook-lint/issues/54)) ([cc0d0cc](https://github.com/joshrotenberg/mdbook-lint/commit/cc0d0cc69d48628dbd9394e7016a5745211fe6df))
* implement MDBOOK005 rule for orphaned file detection ([#58](https://github.com/joshrotenberg/mdbook-lint/issues/58)) ([7bd7582](https://github.com/joshrotenberg/mdbook-lint/commit/7bd75826bf2a8014d42516c6448db125701919ef))
* implement MDBOOK006 rule for internal cross-reference validation ([#59](https://github.com/joshrotenberg/mdbook-lint/issues/59)) ([a35c640](https://github.com/joshrotenberg/mdbook-lint/commit/a35c640dee9794bc675e406db696e27bad7ecad9))
* implement MDBOOK007 rule for include file validation ([#60](https://github.com/joshrotenberg/mdbook-lint/issues/60)) ([dfb97d8](https://github.com/joshrotenberg/mdbook-lint/commit/dfb97d848448647b979e20a9096173e31ce8b847))


### Bug Fixes

* resolve corpus testing markdownlint integration ([#49](https://github.com/joshrotenberg/mdbook-lint/issues/49)) ([bc13242](https://github.com/joshrotenberg/mdbook-lint/commit/bc13242a62809841020371b63d86ef6dfb3e0728))

## [0.3.0](https://github.com/joshrotenberg/mdbook-lint/compare/v0.2.0...v0.3.0) (2025-08-05)


### Features

* add prebuilt binaries for multiple platforms ([#47](https://github.com/joshrotenberg/mdbook-lint/issues/47)) ([8933513](https://github.com/joshrotenberg/mdbook-lint/commit/89335131fcfac4d2bfcdf458814b70aa3c7b9c8f))

## [0.2.0](https://github.com/joshrotenberg/mdbook-lint/compare/v0.1.0...v0.2.0) (2025-08-05)


### Features

* add markdownlint compatibility mode ([#40](https://github.com/joshrotenberg/mdbook-lint/issues/40)) ([4f99765](https://github.com/joshrotenberg/mdbook-lint/commit/4f9976538cb275fc768e1f4355123174048a4874))


### Bug Fixes

* **md044:** resolve Unicode panic with emoji characters ([#41](https://github.com/joshrotenberg/mdbook-lint/issues/41)) ([192ad79](https://github.com/joshrotenberg/mdbook-lint/commit/192ad796ae577b1a716783513b5cdf8bb1a01748))
* **rules:** eliminate duplicate violations between MDBOOK and standard MD rules ([#38](https://github.com/joshrotenberg/mdbook-lint/issues/38)) ([b04b74f](https://github.com/joshrotenberg/mdbook-lint/commit/b04b74f74ff8c5ee495d8a0a3801c3de214f163b))
* **rules:** prevent MD030 from flagging bold text as list markers ([#35](https://github.com/joshrotenberg/mdbook-lint/issues/35)) ([83bf032](https://github.com/joshrotenberg/mdbook-lint/commit/83bf032f79601fa9c4f4eb1ba72ceeb2bf8c3ab5))
* **rules:** prevent MD044 from flagging proper names in URL contexts ([#37](https://github.com/joshrotenberg/mdbook-lint/issues/37)) ([8276772](https://github.com/joshrotenberg/mdbook-lint/commit/8276772fcbfdc8615adf433fcd919518bc001d87)), closes [#21](https://github.com/joshrotenberg/mdbook-lint/issues/21)

## 0.1.0 (2025-08-04)


### Features

* improve test coverage with comprehensive edge case testing ([#7](https://github.com/joshrotenberg/mdbook-lint/issues/7)) ([5ffe072](https://github.com/joshrotenberg/mdbook-lint/commit/5ffe07296703d185b2516652aa2d2fe00340a901))


### Bug Fixes

* **ci:** release please permissions ([#4](https://github.com/joshrotenberg/mdbook-lint/issues/4)) ([fa86ff9](https://github.com/joshrotenberg/mdbook-lint/commit/fa86ff950a31e81765ca51beaf9280c0427d3f91))
* **ci:** resolve CI failures and establish project governance ([#1](https://github.com/joshrotenberg/mdbook-lint/issues/1)) ([a81a990](https://github.com/joshrotenberg/mdbook-lint/commit/a81a990ec68b1cf6b92895ff3ec2fc1c4d9fd0a5))
* **ci:** resolve GitHub Pages deployment configuration ([#6](https://github.com/joshrotenberg/mdbook-lint/issues/6)) ([c9a1c0a](https://github.com/joshrotenberg/mdbook-lint/commit/c9a1c0ae7ff30729e186a6d23c5c4071c8d7ad28))
* **ci:** simplify release-please configuration ([#2](https://github.com/joshrotenberg/mdbook-lint/issues/2)) ([4b65b76](https://github.com/joshrotenberg/mdbook-lint/commit/4b65b766a45ceb752182cff5445572fd2d9e0d89))
