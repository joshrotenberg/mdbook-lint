# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.14.3] - 2026-02-02

### Documentation
- Update Docker version examples to 0.14.2 ([#388](https://github.com/joshrotenberg/mdbook-lint/pull/388)) ([479d545](https://github.com/joshrotenberg/mdbook-lint/commit/479d5452d475b69b0b6f5f5038dbdf30addf401b))



## [0.14.2] - 2026-02-02

### Bug Fixes
- Update Dockerfile for Rust edition 2024 and add Docker docs ([#387](https://github.com/joshrotenberg/mdbook-lint/pull/387)) ([b6aa86b](https://github.com/joshrotenberg/mdbook-lint/commit/b6aa86b45261c2cbf89ec8c313cf0bdda4502f16))



## [0.14.1] - 2026-02-01

### Bug Fixes
- Handle newline duplication in apply_fix methods ([#381](https://github.com/joshrotenberg/mdbook-lint/pull/381)) ([a60c10e](https://github.com/joshrotenberg/mdbook-lint/commit/a60c10eccf72610f88cce37fc7ecb71ef436ae01))



## [0.14.0] - 2026-02-01

### Bug Fixes
- Mdbook 0.5.x compatibility fixes ([#370](https://github.com/joshrotenberg/mdbook-lint/pull/370)) ([327751d](https://github.com/joshrotenberg/mdbook-lint/commit/327751dde8522de998ab9acd854b0646c8bc39ec))
- Add missing blank line in adr003.rs rustdoc ([#376](https://github.com/joshrotenberg/mdbook-lint/pull/376)) ([3544add](https://github.com/joshrotenberg/mdbook-lint/commit/3544addcbefb8ce7b1ead7224996c19825655d69))
- Correct markdown formatting in ADR rustdoc comments ([#375](https://github.com/joshrotenberg/mdbook-lint/pull/375)) ([153f675](https://github.com/joshrotenberg/mdbook-lint/commit/153f675066eeccb0ac060f57add0d243362be0c4))


### Features
- Improve library ergonomics for external consumers ([#372](https://github.com/joshrotenberg/mdbook-lint/pull/372)) ([c959795](https://github.com/joshrotenberg/mdbook-lint/commit/c959795406c6107b7351d40fac68315fbf71d388))


### Testing
- Add comprehensive mdbook 0.4/0.5 compatibility tests ([#374](https://github.com/joshrotenberg/mdbook-lint/pull/374)) ([35bf1d1](https://github.com/joshrotenberg/mdbook-lint/commit/35bf1d14d5888e5e7c5008c8aa683b4f47315e83))



## [0.13.7](https://github.com/joshrotenberg/mdbook-lint/compare/v0.13.6...v0.13.7) - 2026-01-25

### Added

- add rustdoc subcommand to lint //! module docs ([#352](https://github.com/joshrotenberg/mdbook-lint/pull/352))
- add ADR collection rules (ADR010-ADR013) ([#362](https://github.com/joshrotenberg/mdbook-lint/pull/362))
- add ADR content quality rules (ADR014-ADR017) ([#363](https://github.com/joshrotenberg/mdbook-lint/pull/363))
- add ADR rules batch 2 (ADR004-ADR009) ([#361](https://github.com/joshrotenberg/mdbook-lint/pull/361))
- add ADR ruleset for Architecture Decision Records ([#360](https://github.com/joshrotenberg/mdbook-lint/pull/360))

### Fixed

- correct slug generation to match mdBook behavior ([#358](https://github.com/joshrotenberg/mdbook-lint/pull/358))

### Other

- add comprehensive ADR ruleset documentation ([#364](https://github.com/joshrotenberg/mdbook-lint/pull/364))
- add property tests for slug generation ([#359](https://github.com/joshrotenberg/mdbook-lint/pull/359))
- add edge case tests for MD002 first-heading-h1 rule ([#351](https://github.com/joshrotenberg/mdbook-lint/pull/351))
- add edge case tests for MD001 heading-increment rule ([#350](https://github.com/joshrotenberg/mdbook-lint/pull/350))

## [0.13.6](https://github.com/joshrotenberg/mdbook-lint/compare/v0.13.5...v0.13.6) - 2025-12-23

### Fixed

- recognize reference definitions inside blockquotes in MD052 ([#348](https://github.com/joshrotenberg/mdbook-lint/pull/348))
- implement code_blocks config option for MD010 ([#347](https://github.com/joshrotenberg/mdbook-lint/pull/347))
- skip IPA phonetic notation in MD052 reference link checks ([#343](https://github.com/joshrotenberg/mdbook-lint/pull/343))

## [0.13.5](https://github.com/joshrotenberg/mdbook-lint/compare/v0.13.4...v0.13.5) - 2025-12-13

### Fixed

- Reduce false positives in MDBOOK002, CONTENT001, and MD044 ([#340](https://github.com/joshrotenberg/mdbook-lint/pull/340))
- Add support for extended checkbox states, wiki-links, and template variables ([#335](https://github.com/joshrotenberg/mdbook-lint/pull/335))
- Resolve false positives for extended markdown syntax ([#334](https://github.com/joshrotenberg/mdbook-lint/pull/334))
- Support named anchors and add comprehensive test fixtures ([#333](https://github.com/joshrotenberg/mdbook-lint/pull/333))
- Skip bracket patterns in headings for MD052 ([#332](https://github.com/joshrotenberg/mdbook-lint/pull/332))
- Skip display math blocks in MD030 list marker detection ([#331](https://github.com/joshrotenberg/mdbook-lint/pull/331))
- Exclude footnote references from MD011 reversed link detection ([#325](https://github.com/joshrotenberg/mdbook-lint/pull/325))
- Align slug generation with mdBook behavior ([#329](https://github.com/joshrotenberg/mdbook-lint/pull/329))
- Skip link detection inside math blocks in MD052 ([#327](https://github.com/joshrotenberg/mdbook-lint/pull/327))
- Handle escaped dollar signs in MDBOOK010 KaTeX validation ([#326](https://github.com/joshrotenberg/mdbook-lint/pull/326))

### Other

- Add preprocessor to keywords and description ([#336](https://github.com/joshrotenberg/mdbook-lint/pull/336))
- Add comprehensive KaTeX math test fixture ([#330](https://github.com/joshrotenberg/mdbook-lint/pull/330))

## [0.13.4](https://github.com/joshrotenberg/mdbook-lint/compare/v0.13.3...v0.13.4) - 2025-12-12

### Fixed

- preprocessor now discovers .mdbook-lint.toml config files ([#319](https://github.com/joshrotenberg/mdbook-lint/pull/319))

### Other

- fix preprocessor naming from mdbook-lint to lint ([#316](https://github.com/joshrotenberg/mdbook-lint/pull/316))

## [0.13.3](https://github.com/joshrotenberg/mdbook-lint/compare/v0.13.2...v0.13.3) - 2025-12-11

### Added

- add fix subcommand for easier auto-fix discoverability ([#310](https://github.com/joshrotenberg/mdbook-lint/pull/310))

### Bug Fixes
- Enable changelog generation for GitHub release notes ([098b2f6](https://github.com/joshrotenberg/mdbook-lint/commit/098b2f6c4f6bdb34433acdcbd753db6d305b8837))

## [0.13.2] - 2025-12-11

### Bug Fixes
- Strip null values from mdbook JSON to fix TOML deserialization ([#303](https://github.com/joshrotenberg/mdbook-lint/pull/303)) ([9688b10](https://github.com/joshrotenberg/mdbook-lint/commit/9688b10ce262ea7248345b88d3ac9866c148d561))


### CI/CD
- Add mdbook compatibility matrix workflow ([#307](https://github.com/joshrotenberg/mdbook-lint/pull/307)) ([2fc326f](https://github.com/joshrotenberg/mdbook-lint/commit/2fc326f22ab55bc002d8cceef53a9407ef06a55c))


### Documentation
- Add mdbook explanation and version compatibility documentation ([#308](https://github.com/joshrotenberg/mdbook-lint/pull/308)) ([2408ce4](https://github.com/joshrotenberg/mdbook-lint/commit/2408ce4a21f78d44b4b1968675cacfbed55dee84))
- Remove duplicate sections from contributing guide ([#302](https://github.com/joshrotenberg/mdbook-lint/pull/302)) ([a7de4c2](https://github.com/joshrotenberg/mdbook-lint/commit/a7de4c26f843f337a7e45b66fcc6120633ef45bb))


### Features
- Add support for mdbook 0.5.x JSON format ([#305](https://github.com/joshrotenberg/mdbook-lint/pull/305)) ([0581078](https://github.com/joshrotenberg/mdbook-lint/commit/05810783da50e643db3593e4bf9571a494b7c8d0))


### Miscellaneous
- Release v0.13.2 ([#306](https://github.com/joshrotenberg/mdbook-lint/pull/306)) ([c565601](https://github.com/joshrotenberg/mdbook-lint/commit/c5656016d241262d91600999a3a6576ac37c76e1))

## [0.13.1] - 2025-12-07

### Bug Fixes
- Enable content feature by default in CLI ([#293](https://github.com/joshrotenberg/mdbook-lint/pull/293)) ([b6da580](https://github.com/joshrotenberg/mdbook-lint/commit/b6da5808d47d671853430678217805053aadc59a))


### Documentation
- Update rule counts to 83 (10 content rules) ([#298](https://github.com/joshrotenberg/mdbook-lint/pull/298)) ([dda8d80](https://github.com/joshrotenberg/mdbook-lint/commit/dda8d8037f5304569143f5453ad9f52b368a4813))


### Features
- Add five new content quality rules ([#296](https://github.com/joshrotenberg/mdbook-lint/pull/296)) ([e20d5e0](https://github.com/joshrotenberg/mdbook-lint/commit/e20d5e0157dc8391109416c9963b643810faa304))
- Add verbose and quiet CLI flags with colored help ([#295](https://github.com/joshrotenberg/mdbook-lint/pull/295)) ([1228768](https://github.com/joshrotenberg/mdbook-lint/commit/122876877442cfd4fa06668196433084f84b99c2))


### Miscellaneous
- Release v0.13.1 ([#297](https://github.com/joshrotenberg/mdbook-lint/pull/297)) ([7cf1fe0](https://github.com/joshrotenberg/mdbook-lint/commit/7cf1fe02562ed568e81ce5419271a835f912394b))
- Add changelog configuration for release-plz ([#299](https://github.com/joshrotenberg/mdbook-lint/pull/299)) ([16ea997](https://github.com/joshrotenberg/mdbook-lint/commit/16ea9971760243948a900ecb647854ec09097f29))

## [0.13.0] - 2025-12-05

### Bug Fixes
- Move example config into CLI crate for packaging ([#289](https://github.com/joshrotenberg/mdbook-lint/pull/289)) ([722f211](https://github.com/joshrotenberg/mdbook-lint/commit/722f2113076ad93503995feaa6f3e6222aff31bd))


### CI/CD
- Add workflow_dispatch trigger to release workflow ([#290](https://github.com/joshrotenberg/mdbook-lint/pull/290)) ([ae66100](https://github.com/joshrotenberg/mdbook-lint/commit/ae6610053cb139532455fbf5f4937e1b6012a1dd))


### Documentation
- Add acknowledgments for markdownlint and rumdl ([#287](https://github.com/joshrotenberg/mdbook-lint/pull/287)) ([408f574](https://github.com/joshrotenberg/mdbook-lint/commit/408f57441537e7a4cbd69f05b94b8aa02f39cc43))


### Features
- Add visual line length mode to MD013 ([#291](https://github.com/joshrotenberg/mdbook-lint/pull/291)) ([997ddf4](https://github.com/joshrotenberg/mdbook-lint/commit/997ddf4c7910389fa7fe6dc17efd602b516b2e6d))
- Add cargo-style colored output ([#288](https://github.com/joshrotenberg/mdbook-lint/pull/288)) ([df5d267](https://github.com/joshrotenberg/mdbook-lint/commit/df5d267045b74b1f7fe85b18b2690e9778894467))
- Add CLI content feature passthrough ([#285](https://github.com/joshrotenberg/mdbook-lint/pull/285)) ([43fbcc0](https://github.com/joshrotenberg/mdbook-lint/commit/43fbcc0b88fa47f6dac2b6be92d8be2f4a2876e2))


### Miscellaneous
- Release v0.13.0 ([#292](https://github.com/joshrotenberg/mdbook-lint/pull/292)) ([074fc08](https://github.com/joshrotenberg/mdbook-lint/commit/074fc08fc7e0e347468a479d282e4cd5226c7819))
- Release v0.12.1 ([#286](https://github.com/joshrotenberg/mdbook-lint/pull/286)) ([33c8231](https://github.com/joshrotenberg/mdbook-lint/commit/33c8231b29f27ab0a38ff277a5c9256fa059e43d))

## [0.12.0] - 2025-12-05

### Bug Fixes
- MD033 and MD026 false positive improvements ([#282](https://github.com/joshrotenberg/mdbook-lint/pull/282)) ([bf92206](https://github.com/joshrotenberg/mdbook-lint/commit/bf922069cb2afbcd2eb19862b22a20a5c6ff97fc))
- Resolve false positives in multiple rules ([#279](https://github.com/joshrotenberg/mdbook-lint/pull/279)) ([842d57a](https://github.com/joshrotenberg/mdbook-lint/commit/842d57a1b851e071adaa0ffd4e585452ab31f9d2))
- Enable GitHub release for mdbook-lint package ([#255](https://github.com/joshrotenberg/mdbook-lint/pull/255)) ([8b5a60c](https://github.com/joshrotenberg/mdbook-lint/commit/8b5a60c9eeb790e9defcd30e5441692d27329dcc))


### Features
- Add content feature flag for CONTENT rules ([#284](https://github.com/joshrotenberg/mdbook-lint/pull/284)) ([e6a9331](https://github.com/joshrotenberg/mdbook-lint/commit/e6a93317242575047dfb25da7fc4f2f9a190a942))
- Improve rules command output formatting ([#266](https://github.com/joshrotenberg/mdbook-lint/pull/266)) ([cc99808](https://github.com/joshrotenberg/mdbook-lint/commit/cc9980814a9338c29593abfe59678b2b64c03c8b))
- Deprecate MD006 and add MD060 (table column style) ([#262](https://github.com/joshrotenberg/mdbook-lint/pull/262)) ([a98440e](https://github.com/joshrotenberg/mdbook-lint/commit/a98440e9906e19f36ee86980a4cd56181c1c32e8))
- Add markdownlint comparison script and fix corpus testing workflow ([#261](https://github.com/joshrotenberg/mdbook-lint/pull/261)) ([2ac7302](https://github.com/joshrotenberg/mdbook-lint/commit/2ac730206e368df842e29d2f60448c3077499fbc))
- Add per-rule auto-fix configuration ([#260](https://github.com/joshrotenberg/mdbook-lint/pull/260)) ([0b8970c](https://github.com/joshrotenberg/mdbook-lint/commit/0b8970c406238622bac5943952078d91306e39fe))
- Add MDBOOK016 and MDBOOK017 rules ([#259](https://github.com/joshrotenberg/mdbook-lint/pull/259)) ([bff7bbf](https://github.com/joshrotenberg/mdbook-lint/commit/bff7bbf51037a4de19dcec445f5ba4c5d5e655a5))
- Add CONTENT003, CONTENT004, CONTENT005 rules ([#258](https://github.com/joshrotenberg/mdbook-lint/pull/258)) ([d764154](https://github.com/joshrotenberg/mdbook-lint/commit/d7641541eb56e775a01e46fb1ccbdf65988c9336))
- Add CONTENT002, MDBOOK021, MDBOOK022 rules ([#257](https://github.com/joshrotenberg/mdbook-lint/pull/257)) ([82cac2d](https://github.com/joshrotenberg/mdbook-lint/commit/82cac2dfd3b64aa5e59323d96edb103284a6666d))
- Add stdin support for lint command ([#254](https://github.com/joshrotenberg/mdbook-lint/pull/254)) ([ebaa3c4](https://github.com/joshrotenberg/mdbook-lint/commit/ebaa3c4638ba9f4106760bd7647d11c051cf6894))


### Miscellaneous
- Release v0.12.0 ([#253](https://github.com/joshrotenberg/mdbook-lint/pull/253)) ([567d805](https://github.com/joshrotenberg/mdbook-lint/commit/567d80531cac1d50714075d9374aad03574b70b4))


### Performance
- Parallelize file linting with rayon ([#256](https://github.com/joshrotenberg/mdbook-lint/pull/256)) ([725354f](https://github.com/joshrotenberg/mdbook-lint/commit/725354fdb87066d9ccf51e9553c306a5f1af8ece))

## [0.11.7] - 2025-12-04

### Bug Fixes
- Apply auto-fix to root markdown files ([#247](https://github.com/joshrotenberg/mdbook-lint/pull/247)) ([7c7f3cb](https://github.com/joshrotenberg/mdbook-lint/commit/7c7f3cbbc186ebbe7db482e43017ea7c030895cf))
- Increase MD051 performance test threshold to 150ms ([#249](https://github.com/joshrotenberg/mdbook-lint/pull/249)) ([aed3dde](https://github.com/joshrotenberg/mdbook-lint/commit/aed3ddebf2f58dc528c2ae71b49850a7f5fdaac9))
- MD044 should not flag proper names in markdown link URLs ([#248](https://github.com/joshrotenberg/mdbook-lint/pull/248)) ([10636e3](https://github.com/joshrotenberg/mdbook-lint/commit/10636e359b14ef3af00e0ae4049ce00e01486dbf))
- Resolve .html links to .md files in MDBOOK002 ([#244](https://github.com/joshrotenberg/mdbook-lint/pull/244)) ([d3555af](https://github.com/joshrotenberg/mdbook-lint/commit/d3555af211a1d847763695b25fd771bcc3eb9e21))
- Apply auto-fix to documentation and fix broken links ([#245](https://github.com/joshrotenberg/mdbook-lint/pull/245)) ([392ed5a](https://github.com/joshrotenberg/mdbook-lint/commit/392ed5a710f3d6ddd044330dd1c3d194b1b0b7ac))
- Re-enable disabled mdBook rule tests ([#241](https://github.com/joshrotenberg/mdbook-lint/pull/241)) ([df813b3](https://github.com/joshrotenberg/mdbook-lint/commit/df813b3e55cca86a0162486c9daa152732c7d3d2))


### Documentation
- Complete individual rule documentation pages ([#251](https://github.com/joshrotenberg/mdbook-lint/pull/251)) ([3370e4e](https://github.com/joshrotenberg/mdbook-lint/commit/3370e4e8a779fcf6c019418cfa6c0f349fdb8f0c))
- Clarify CLI vs preprocessor recommendations ([#242](https://github.com/joshrotenberg/mdbook-lint/pull/242)) ([b5d6e76](https://github.com/joshrotenberg/mdbook-lint/commit/b5d6e76016c24dd56ac11b8482f1950c895e69a4))


### Features
- Add MDBOOK023 and CONTENT001 rules ([#252](https://github.com/joshrotenberg/mdbook-lint/pull/252)) ([e02ee61](https://github.com/joshrotenberg/mdbook-lint/commit/e02ee617cc78dd080f0ca566766031fd4519299d))
- Add rule name and category validation to check command ([#250](https://github.com/joshrotenberg/mdbook-lint/pull/250)) ([2a638a0](https://github.com/joshrotenberg/mdbook-lint/commit/2a638a06b107f27db325ee39981577a969ea98a6))
- Add smart CLI argument detection ([#240](https://github.com/joshrotenberg/mdbook-lint/pull/240)) ([7a02ffb](https://github.com/joshrotenberg/mdbook-lint/commit/7a02ffb7b4a4c938c35318215865be38e9285661))


### Miscellaneous
- Release v0.11.7 ([#243](https://github.com/joshrotenberg/mdbook-lint/pull/243)) ([ecd164b](https://github.com/joshrotenberg/mdbook-lint/commit/ecd164b1ab2406ee3096b74c8893453f3e43ad91))

## [0.11.6] - 2025-09-02

### Bug Fixes
- MD034 now correctly skips reference link definitions ([#237](https://github.com/joshrotenberg/mdbook-lint/pull/237)) ([7b0c66a](https://github.com/joshrotenberg/mdbook-lint/commit/7b0c66ae962694af12147fdecd871197352bbb57))
- MD023 now properly skips code blocks using AST detection ([#236](https://github.com/joshrotenberg/mdbook-lint/pull/236)) ([86e1e73](https://github.com/joshrotenberg/mdbook-lint/commit/86e1e739a21e885c7007f4ec86020539c2392f55))
- MD034 now properly skips code blocks using AST detection ([#235](https://github.com/joshrotenberg/mdbook-lint/pull/235)) ([5ce9b9a](https://github.com/joshrotenberg/mdbook-lint/commit/5ce9b9a44948f8cbb54c403cc9aa4ab1f414f7d9))
- MD007 now correctly ignores code blocks ([#228](https://github.com/joshrotenberg/mdbook-lint/pull/228)) ([4c68bd8](https://github.com/joshrotenberg/mdbook-lint/commit/4c68bd862a17fbb51bff9519082e9acfc99277f0))
- Apply auto-fixes to documentation ([#204](https://github.com/joshrotenberg/mdbook-lint/pull/204)) ([dcda9f7](https://github.com/joshrotenberg/mdbook-lint/commit/dcda9f7fdf41e84134a8d560e2315becbc4b9878))


### Documentation
- Update rules reference with all 41 fixable rules ([#226](https://github.com/joshrotenberg/mdbook-lint/pull/226)) ([23e6b74](https://github.com/joshrotenberg/mdbook-lint/commit/23e6b74d6263a6a58fcef8e25c8ff88f28ddacd5))
- Add tool comparison analysis (mdbook-lint vs markdownlint) ([#227](https://github.com/joshrotenberg/mdbook-lint/pull/227)) ([8bccf08](https://github.com/joshrotenberg/mdbook-lint/commit/8bccf0805d87d84cbcf19944cdb8440cd3d96613))
- Update rules reference with complete fixable rules list ([#221](https://github.com/joshrotenberg/mdbook-lint/pull/221)) ([1e20f34](https://github.com/joshrotenberg/mdbook-lint/commit/1e20f34cd5bb2ec69f4573828558e6580e4f9eb2))
- Add CI vs preprocessor integration guide ([#196](https://github.com/joshrotenberg/mdbook-lint/pull/196)) ([d04c39d](https://github.com/joshrotenberg/mdbook-lint/commit/d04c39d2e82e11ac8277099aa62ee19320473fc9))


### Features
- Add auto-fix for MD058 (tables surrounded by blank lines) ([#225](https://github.com/joshrotenberg/mdbook-lint/pull/225)) ([afb827f](https://github.com/joshrotenberg/mdbook-lint/commit/afb827fd520fbdd1b0de37cc4e96bdb96ff8f6a4))
- Add auto-fix for MD049 and MD046 ([#224](https://github.com/joshrotenberg/mdbook-lint/pull/224)) ([820877a](https://github.com/joshrotenberg/mdbook-lint/commit/820877a9d182bdc8eef5f5a125ecd90dd7a01188))
- Add auto-fix for formatting rules (MD014, MD028, MD035, MD045) ([#218](https://github.com/joshrotenberg/mdbook-lint/pull/218)) ([8cd9409](https://github.com/joshrotenberg/mdbook-lint/commit/8cd9409c8a2c483cf92aaf70f8bfa49977485efc))
- Add auto-fix for MD048 and MD050 ([#219](https://github.com/joshrotenberg/mdbook-lint/pull/219)) ([39a161c](https://github.com/joshrotenberg/mdbook-lint/commit/39a161c1f10c0b45d6c20157fcc71bb248ea9ea0))
- Add auto-fix for table rules (MD055, MD056) ([#220](https://github.com/joshrotenberg/mdbook-lint/pull/220)) ([6683042](https://github.com/joshrotenberg/mdbook-lint/commit/668304296397696ba44c15d1f5f33ed25a8704df))
- Implement auto-fix for MD029 (ordered list prefix consistency) ([#215](https://github.com/joshrotenberg/mdbook-lint/pull/215)) ([a75d37b](https://github.com/joshrotenberg/mdbook-lint/commit/a75d37b0bd9b67ecd57ab72a70c2b4bb44c807b9))
- Implement auto-fix for heading rules (MD001, MD002, MD024, MD025, MD026) ([#213](https://github.com/joshrotenberg/mdbook-lint/pull/213)) ([a9d28ef](https://github.com/joshrotenberg/mdbook-lint/commit/a9d28efb99858afd5b41535ccaa97d7757ba115c))
- Enable can_fix() for 8 rules with existing fix implementations ([#212](https://github.com/joshrotenberg/mdbook-lint/pull/212)) ([e2ebc3a](https://github.com/joshrotenberg/mdbook-lint/commit/e2ebc3a7faa2493ea7c7f3d653c36e2712c33acd))
- Enable auto-fix for MD009, MD012, MD027 whitespace rules ([#211](https://github.com/joshrotenberg/mdbook-lint/pull/211)) ([04189da](https://github.com/joshrotenberg/mdbook-lint/commit/04189da52c8c877da30ca2d0468ee6ce2982860f))
- Implement auto-fix for MD037, MD038, MD039 ([#208](https://github.com/joshrotenberg/mdbook-lint/pull/208)) ([fc7b952](https://github.com/joshrotenberg/mdbook-lint/commit/fc7b952fa0a0646b950e94f0b257e09376350d47))
- Add auto-fix support for MD031 and MD032 ([#207](https://github.com/joshrotenberg/mdbook-lint/pull/207)) ([72811ad](https://github.com/joshrotenberg/mdbook-lint/commit/72811ad1e5259ee7528de7336d05b2d7a038d65a))
- Enable auto-fix for MD018, MD019, MD020 ATX heading rules ([#209](https://github.com/joshrotenberg/mdbook-lint/pull/209)) ([9b00cc8](https://github.com/joshrotenberg/mdbook-lint/commit/9b00cc8fcb747fdf78c18dab2089a87772743b48))
- Add auto-fix support for MD005, MD006, and MD007 ([#206](https://github.com/joshrotenberg/mdbook-lint/pull/206)) ([6a61659](https://github.com/joshrotenberg/mdbook-lint/commit/6a61659745cfad222f9a408cafb229dd7e2fa8b0))
- Phase 1 auto-fix batch - MD004 and MD011 ([#201](https://github.com/joshrotenberg/mdbook-lint/pull/201)) ([9f8cd8a](https://github.com/joshrotenberg/mdbook-lint/commit/9f8cd8a21ac0ded01381d6c807b21ac88bd0f8e7))
- Implement auto-fix for MD003 (heading-style) ([#199](https://github.com/joshrotenberg/mdbook-lint/pull/199)) ([d399acd](https://github.com/joshrotenberg/mdbook-lint/commit/d399acd6ff47f889daed3f34ba01379a38914d82))
- Implement auto-fix for MD022 (blanks-around-headings) ([#198](https://github.com/joshrotenberg/mdbook-lint/pull/198)) ([60a84e2](https://github.com/joshrotenberg/mdbook-lint/commit/60a84e2ede47af06e6e7903ddc2e747dc1437487))


### Miscellaneous
- Release v0.11.6 ([#234](https://github.com/joshrotenberg/mdbook-lint/pull/234)) ([6259078](https://github.com/joshrotenberg/mdbook-lint/commit/625907835d11581a6b094c63a7f64661ff06bf36))


### Testing
- Add comprehensive auto-fix test coverage ([#205](https://github.com/joshrotenberg/mdbook-lint/pull/205)) ([6cad689](https://github.com/joshrotenberg/mdbook-lint/commit/6cad6895bacb0a312fede1ef4b10200172140302))
- Improve auto-fix test coverage ([#203](https://github.com/joshrotenberg/mdbook-lint/pull/203)) ([3372f3d](https://github.com/joshrotenberg/mdbook-lint/commit/3372f3dcf7b2dda0986441faf640926fc9abc80d))

## [0.11.5] - 2025-08-29

### Bug Fixes
- Resolve release-plz workspace dependency issues ([#194](https://github.com/joshrotenberg/mdbook-lint/pull/194)) ([89a15b7](https://github.com/joshrotenberg/mdbook-lint/commit/89a15b703f2c36077373a5249d9db53b50ac64fc))
- Correct rule configuration pattern in documentation ([#192](https://github.com/joshrotenberg/mdbook-lint/pull/192)) ([86ab737](https://github.com/joshrotenberg/mdbook-lint/commit/86ab737936f22bb5e3de16e767ddc9ecd8d13262))


### Miscellaneous
- Release v0.11.5 ([#193](https://github.com/joshrotenberg/mdbook-lint/pull/193)) ([d1a8ce1](https://github.com/joshrotenberg/mdbook-lint/commit/d1a8ce17ce29e56b7ba048cb8a5219c00b9823ea))

## [0.11.4] - 2025-08-29

### Bug Fixes
- Preprocessor now properly uses discovered configuration ([#185](https://github.com/joshrotenberg/mdbook-lint/pull/185)) ([a11e788](https://github.com/joshrotenberg/mdbook-lint/commit/a11e788e323f3df46126d1a2513e13f6bb003d3a))
- Handle [rules] section with default=false in TOML config ([#181](https://github.com/joshrotenberg/mdbook-lint/pull/181)) ([6b61763](https://github.com/joshrotenberg/mdbook-lint/commit/6b6176383737fe71fa01e4e03bb45f70236e2854))
- MDBOOK002 correctly resolve absolute paths relative to book source directory ([#168](https://github.com/joshrotenberg/mdbook-lint/pull/168)) ([5bd70b0](https://github.com/joshrotenberg/mdbook-lint/commit/5bd70b02fc24487b9b4015c913c0b2659f09b361))
- Improve MD047 error messages for clarity ([#165](https://github.com/joshrotenberg/mdbook-lint/pull/165)) ([240bc41](https://github.com/joshrotenberg/mdbook-lint/commit/240bc41d8cc341445852b90152a527877d4e0f1c))
- Configure release-plz to avoid tag conflicts ([#154](https://github.com/joshrotenberg/mdbook-lint/pull/154)) ([52b4e9b](https://github.com/joshrotenberg/mdbook-lint/commit/52b4e9b82f47d0a81772b661c516d57087ed7ef9))


### Documentation
- Final documentation updates for v0.11.3 ([#188](https://github.com/joshrotenberg/mdbook-lint/pull/188)) ([c19a761](https://github.com/joshrotenberg/mdbook-lint/commit/c19a761bf255a0ae4e33b5d32211d23d568e8336))
- Comprehensive update to configuration documentation ([#187](https://github.com/joshrotenberg/mdbook-lint/pull/187)) ([ab4f24b](https://github.com/joshrotenberg/mdbook-lint/commit/ab4f24b70647e179249e5071644d1eca1ee4f458))


### Features
- Implement auto-fix support for all rules with fixes ([#186](https://github.com/joshrotenberg/mdbook-lint/pull/186)) ([6b59e40](https://github.com/joshrotenberg/mdbook-lint/commit/6b59e403bb6d59964e7260eef468fcc975a4d92a))
- Implement configuration support for batch 4 rules and complete rule configuration ([#179](https://github.com/joshrotenberg/mdbook-lint/pull/179)) ([93f8e26](https://github.com/joshrotenberg/mdbook-lint/commit/93f8e267e5a2820a44355f5748c765db82f60423))
- Implement configuration support for batch 3 rules (MD035, MD036, MD043, MD044, MD046) ([#178](https://github.com/joshrotenberg/mdbook-lint/pull/178)) ([b73615c](https://github.com/joshrotenberg/mdbook-lint/commit/b73615c26d7a999cc0e1b5d07d8a99717132ce80))
- Add configuration support for batch 2 rules (MD024-MD030) ([#177](https://github.com/joshrotenberg/mdbook-lint/pull/177)) ([d263587](https://github.com/joshrotenberg/mdbook-lint/commit/d263587625060249f5bbf5f2cbc86659c585030a))
- Implement configuration support for batch 1 rules (MD002, MD003, MD007, MD010, MD012) ([#176](https://github.com/joshrotenberg/mdbook-lint/pull/176)) ([fafae87](https://github.com/joshrotenberg/mdbook-lint/commit/fafae87eea80b9984d7d5e0408b7ccf8e9e61006))
- Improve MDBOOK001 to clarify plain text code block support ([#166](https://github.com/joshrotenberg/mdbook-lint/pull/166)) ([4992c14](https://github.com/joshrotenberg/mdbook-lint/commit/4992c146cec8868800bab6b5df20f7a519083d3d))
- Implement rule configuration support ([#170](https://github.com/joshrotenberg/mdbook-lint/pull/170)) ([c213b69](https://github.com/joshrotenberg/mdbook-lint/commit/c213b6966d8bd7d82370001a5504bf41e5b3dce6))


### Miscellaneous
- Release v0.11.4 ([#190](https://github.com/joshrotenberg/mdbook-lint/pull/190)) ([6f43776](https://github.com/joshrotenberg/mdbook-lint/commit/6f43776052051cbd3d37d360c17e67754ba40f35))
- Release v0.11.4 ([#167](https://github.com/joshrotenberg/mdbook-lint/pull/167)) ([77bf73e](https://github.com/joshrotenberg/mdbook-lint/commit/77bf73ec90edb3eaf53dfbba99f3d9b2d5b64d4f))
- Update dependencies to latest compatible versions ([#189](https://github.com/joshrotenberg/mdbook-lint/pull/189)) ([796000d](https://github.com/joshrotenberg/mdbook-lint/commit/796000d5e0c130bd7cbd0c54d812bfc89e02f22f))

## [0.11.2] - 2025-08-28

### Bug Fixes
- Exclude integration tests from code coverage to prevent timeouts ([#152](https://github.com/joshrotenberg/mdbook-lint/pull/152)) ([040486b](https://github.com/joshrotenberg/mdbook-lint/commit/040486b4911973c2269e6018eef5369ec3672550))
- Skip math block detection inside code blocks (MDBOOK010) ([#144](https://github.com/joshrotenberg/mdbook-lint/pull/144)) ([5f4f389](https://github.com/joshrotenberg/mdbook-lint/commit/5f4f3897a9f98c2fd650490072e7e018bd1439c6))
- Limit MDBOOK005 orphaned file detection to book source directory ([#145](https://github.com/joshrotenberg/mdbook-lint/pull/145)) ([ae51431](https://github.com/joshrotenberg/mdbook-lint/commit/ae5143198288990ea19e3e5a838287225fb54ece))


### Documentation
- Improve preprocessor documentation and add troubleshooting guide ([#153](https://github.com/joshrotenberg/mdbook-lint/pull/153)) ([2277873](https://github.com/joshrotenberg/mdbook-lint/commit/22778730e73cb4b0721df0eaa022922b68a3b549))


### Features
- Add automatic config file discovery ([#149](https://github.com/joshrotenberg/mdbook-lint/pull/149)) ([e23ff45](https://github.com/joshrotenberg/mdbook-lint/commit/e23ff45c0fec2b68febd9f97a2cd24c9454258f4))


### Miscellaneous
- Release v0.11.2 ([#150](https://github.com/joshrotenberg/mdbook-lint/pull/150)) ([390c2e2](https://github.com/joshrotenberg/mdbook-lint/commit/390c2e24aa2e3fe06888d096afab179f935d014c))


### Testing
- Increase performance test timeout to 100ms ([#146](https://github.com/joshrotenberg/mdbook-lint/pull/146)) ([cfaab28](https://github.com/joshrotenberg/mdbook-lint/commit/cfaab28c15d33a4589f09a5cb0f8ca2cfa7304db))

## [0.11.1] - 2025-08-25

### Features
- Add README to crates.io listing ([#136](https://github.com/joshrotenberg/mdbook-lint/pull/136)) ([996e62a](https://github.com/joshrotenberg/mdbook-lint/commit/996e62a2d96aedb039850aef78218797ffdee0ed))


### Miscellaneous
- Release v0.11.1 ([#139](https://github.com/joshrotenberg/mdbook-lint/pull/139)) ([3bf851e](https://github.com/joshrotenberg/mdbook-lint/commit/3bf851eff91e8dcece6004bed0d21ed5757f94db))

## [0.11.0] - 2025-08-25

### Bug Fixes
- Add docs/.mdbook-lint.toml to gitignore exceptions ([#134](https://github.com/joshrotenberg/mdbook-lint/pull/134)) ([ff74fe9](https://github.com/joshrotenberg/mdbook-lint/commit/ff74fe91175c36ffcdcc9869a901e05d077c5e98))
- Prevent MDBOOK010 from flagging shell prompts as unclosed math ([#124](https://github.com/joshrotenberg/mdbook-lint/pull/124)) ([2465da1](https://github.com/joshrotenberg/mdbook-lint/commit/2465da1b0138e604f3800081140dc38110578134))
- Add example-mdbook-lint.toml to gitignore exceptions ([#122](https://github.com/joshrotenberg/mdbook-lint/pull/122)) ([7f1acac](https://github.com/joshrotenberg/mdbook-lint/commit/7f1acacb4b14c95072208872885d32b741db6ade))
- Handle UTF-8 character boundaries correctly in MD030 rule ([#118](https://github.com/joshrotenberg/mdbook-lint/pull/118)) ([e245192](https://github.com/joshrotenberg/mdbook-lint/commit/e245192adfd6ab8ad9193e55364c7c7e31c585cf))
- Cleanup ([#98](https://github.com/joshrotenberg/mdbook-lint/pull/98)) ([ff7fe96](https://github.com/joshrotenberg/mdbook-lint/commit/ff7fe964088ee516179f41f85545f8256317f419))
- Remove CHANGELOG.md to fix release-plz workflow ([2609fa2](https://github.com/joshrotenberg/mdbook-lint/commit/2609fa25280b88da42f464287e34d2eb9811cbe4))
- Disable changelog updates in release-plz to fix CI ([#94](https://github.com/joshrotenberg/mdbook-lint/pull/94)) ([296e698](https://github.com/joshrotenberg/mdbook-lint/commit/296e6980b7b5c2fed41c99da6bcbc20b48153f41))


### Documentation
- Update README for v0.11.0 release ([#121](https://github.com/joshrotenberg/mdbook-lint/pull/121)) ([e9e2780](https://github.com/joshrotenberg/mdbook-lint/commit/e9e2780c54f33f1ebe267c7ce9185d18e5136617))
- Add complete mdBook rules documentation and example configuration ([#120](https://github.com/joshrotenberg/mdbook-lint/pull/120)) ([3d95b81](https://github.com/joshrotenberg/mdbook-lint/commit/3d95b812b96dbd186ac817f117846d969d012d9d))
- Add comprehensive rule documentation ([#119](https://github.com/joshrotenberg/mdbook-lint/pull/119)) ([c633fef](https://github.com/joshrotenberg/mdbook-lint/commit/c633fef308ad0e0aee0210e78f40e131405ab6a8))
- Comprehensive documentation improvements for API and rules ([#117](https://github.com/joshrotenberg/mdbook-lint/pull/117)) ([0de2259](https://github.com/joshrotenberg/mdbook-lint/commit/0de225918c29134af11361aa89f6ab93a0ff7b40))
- Update documentation for auto-fix functionality ([#116](https://github.com/joshrotenberg/mdbook-lint/pull/116)) ([5ea05d7](https://github.com/joshrotenberg/mdbook-lint/commit/5ea05d72ec30bcef76f6c6b0477a5e5a7627ac1a))
- Comprehensive rustdoc and mdBook documentation restructure ([#112](https://github.com/joshrotenberg/mdbook-lint/pull/112)) ([033442b](https://github.com/joshrotenberg/mdbook-lint/commit/033442bdd4a5305b3e8e6194240df6edb493abc3))
- Add installation verification step ([#93](https://github.com/joshrotenberg/mdbook-lint/pull/93)) ([5b04652](https://github.com/joshrotenberg/mdbook-lint/commit/5b04652ce53e7c4f7dc1bd907e3ad76b448d3ca6))


### Features
- Implement auto-fix functionality for CLI ([#115](https://github.com/joshrotenberg/mdbook-lint/pull/115)) ([d715630](https://github.com/joshrotenberg/mdbook-lint/commit/d7156301c44debc83db030568a1c49f83ba1ce42))
- Implement mdBook rules MDBOOK008-012 ([#114](https://github.com/joshrotenberg/mdbook-lint/pull/114)) ([4210770](https://github.com/joshrotenberg/mdbook-lint/commit/42107703235503133aab4c40cf545621cd452b0b))
- Add fix support for MD030 with comprehensive tests ([#111](https://github.com/joshrotenberg/mdbook-lint/pull/111)) ([0c64b96](https://github.com/joshrotenberg/mdbook-lint/commit/0c64b9607a731e041593dd83db62f440e2206157))
- Add fix support for MD019, MD020, MD021, and MD027 ([#110](https://github.com/joshrotenberg/mdbook-lint/pull/110)) ([1575ba8](https://github.com/joshrotenberg/mdbook-lint/commit/1575ba86a71237a8b4b85623130c57383cc35708))
- Add fix support for MD009, MD012, MD018, MD023, and MD034 ([#109](https://github.com/joshrotenberg/mdbook-lint/pull/109)) ([042e8f1](https://github.com/joshrotenberg/mdbook-lint/commit/042e8f11b2e5cbd66d5fa745b1ff35d724226240))
- Complete rules migration from core to rulesets ([#103](https://github.com/joshrotenberg/mdbook-lint/pull/103)) ([#106](https://github.com/joshrotenberg/mdbook-lint/pull/106)) ([96b2a48](https://github.com/joshrotenberg/mdbook-lint/commit/96b2a483c854b11a835d4819d16292956bec1862))
- Migrate standard rules to rulesets crate ([#105](https://github.com/joshrotenberg/mdbook-lint/pull/105)) ([6dfa806](https://github.com/joshrotenberg/mdbook-lint/commit/6dfa80666c6af7cc8c6b4fa1882df935817a95fb))
- Migrate mdBook rules to rulesets crate (Part 1 of #66) ([#104](https://github.com/joshrotenberg/mdbook-lint/pull/104)) ([a093eea](https://github.com/joshrotenberg/mdbook-lint/commit/a093eead6d50c8c7e825b9503b27d0ddede39ac7))
- Add fix support for MD047 and MD010 rules ([#99](https://github.com/joshrotenberg/mdbook-lint/pull/99)) ([de3694c](https://github.com/joshrotenberg/mdbook-lint/commit/de3694c649b1aa219fb5b117edc9797724e9384c))
- Add automated binary builds for releases ([#96](https://github.com/joshrotenberg/mdbook-lint/pull/96)) ([e25af02](https://github.com/joshrotenberg/mdbook-lint/commit/e25af0275677333fb9e8b7a53c3f17ade96a5d77))


### Miscellaneous
- Remove git-cliff configuration ([#135](https://github.com/joshrotenberg/mdbook-lint/pull/135)) ([eb352ce](https://github.com/joshrotenberg/mdbook-lint/commit/eb352cea3885efa10c090c187604584402695206))
- Release v0.11.0 ([#123](https://github.com/joshrotenberg/mdbook-lint/pull/123)) ([976118a](https://github.com/joshrotenberg/mdbook-lint/commit/976118a80c0ac3c422c0d38a2357e6e71dd4df79))
- Add missing newline at end of Cargo.toml ([#97](https://github.com/joshrotenberg/mdbook-lint/pull/97)) ([f4ef822](https://github.com/joshrotenberg/mdbook-lint/commit/f4ef822f44dbc3276433f3838e9ed9795dbfc655))

## [0.10.0] - 2025-08-15

### Bug Fixes
- Simplify release-plz setup to match standard Rust projects ([#92](https://github.com/joshrotenberg/mdbook-lint/pull/92)) ([507abe9](https://github.com/joshrotenberg/mdbook-lint/commit/507abe987a20bee6b9b02641ace94cc154dc6807))


### Miscellaneous
- Release v0.10.0 ([#91](https://github.com/joshrotenberg/mdbook-lint/pull/91)) ([535351c](https://github.com/joshrotenberg/mdbook-lint/commit/535351c6864770551f12dea67052d904265f6afb))

## [0.9.0] - 2025-08-15

### Features
- Enable crates.io publishing in release-plz ([#89](https://github.com/joshrotenberg/mdbook-lint/pull/89)) ([06a6cd5](https://github.com/joshrotenberg/mdbook-lint/commit/06a6cd566af14993b8bde9327a4e93981b7934c1))
- Add release-plz workflow and configuration ([#86](https://github.com/joshrotenberg/mdbook-lint/pull/86)) ([25ed4b8](https://github.com/joshrotenberg/mdbook-lint/commit/25ed4b8f8c68a8ba73f6b95a9fc36daceffe9add))


### Miscellaneous
- Release v0.9.0 ([#90](https://github.com/joshrotenberg/mdbook-lint/pull/90)) ([f4b84e2](https://github.com/joshrotenberg/mdbook-lint/commit/f4b84e2b22150df40d3ffc86ba4ef251d52bf6a2))
- Release v0.8.0 ([#88](https://github.com/joshrotenberg/mdbook-lint/pull/88)) ([419915b](https://github.com/joshrotenberg/mdbook-lint/commit/419915be39e80bebcb283299e4174e6b3d7ad510))
- Release v0.7.0 ([#87](https://github.com/joshrotenberg/mdbook-lint/pull/87)) ([f24b448](https://github.com/joshrotenberg/mdbook-lint/commit/f24b4486bc1578d6250e2537a2c7ea984904172f))

## [0.6.0] - 2025-08-08

### Miscellaneous
- Release v0.6.0 ([#83](https://github.com/joshrotenberg/mdbook-lint/pull/83)) ([0c101a2](https://github.com/joshrotenberg/mdbook-lint/commit/0c101a2d2b09351f2df7bd964dedb2d3c7bea00a))


### Refactoring
- Consolidate all release workflows into one comprehensive solution ([#84](https://github.com/joshrotenberg/mdbook-lint/pull/84)) ([4d65842](https://github.com/joshrotenberg/mdbook-lint/commit/4d65842e8afeb811523740faec43bdf17aa71ee5))

## [0.5.0] - 2025-08-08

### Bug Fixes
- Configure release-plz to allow dirty working directory ([#80](https://github.com/joshrotenberg/mdbook-lint/pull/80)) ([cc43c96](https://github.com/joshrotenberg/mdbook-lint/commit/cc43c96d5493307e884dfdb5c6daf9d77991f01f))
- Clean up remaining corpus files and improve release-plz workflow ([#79](https://github.com/joshrotenberg/mdbook-lint/pull/79)) ([687f946](https://github.com/joshrotenberg/mdbook-lint/commit/687f9468f80548d6e61cc305f18ab9674745f96d))
- Update mdBook rules help text to reflect current count ([#77](https://github.com/joshrotenberg/mdbook-lint/pull/77)) ([2c12335](https://github.com/joshrotenberg/mdbook-lint/commit/2c123351f6dd041e9e64188ca6077f904566cf2d))


### Features
- Add simple release workflow as alternative to release-plz ([#81](https://github.com/joshrotenberg/mdbook-lint/pull/81)) ([e1fb0c0](https://github.com/joshrotenberg/mdbook-lint/commit/e1fb0c0d666ccc54c073bb16f33f0ee52edf9870))
- Switch from release-please to release-plz for automated releases ([#78](https://github.com/joshrotenberg/mdbook-lint/pull/78)) ([eba62cc](https://github.com/joshrotenberg/mdbook-lint/commit/eba62cc1cd881dda6b9ddc059d25b3d5ac4cb5ff))


### Miscellaneous
- Release v0.5.0 ([#82](https://github.com/joshrotenberg/mdbook-lint/pull/82)) ([8b48e93](https://github.com/joshrotenberg/mdbook-lint/commit/8b48e93a999318ff26c2074366cab59243b5c5c3))

## [0.4.1] - 2025-08-08

### Bug Fixes
- Sync workspace version to 0.4.1 to match tag and release ([#75](https://github.com/joshrotenberg/mdbook-lint/pull/75)) ([66cfffc](https://github.com/joshrotenberg/mdbook-lint/commit/66cfffc374b9ef418b37ac326b0de9f60fe9e1fa))
- Create v0.4.1 tag and sync versions for release-please ([#74](https://github.com/joshrotenberg/mdbook-lint/pull/74)) ([d9e0062](https://github.com/joshrotenberg/mdbook-lint/commit/d9e0062e82e73d83ed46addfce68fdd9c85d1943))
- Reset version to 0.4.0 to sync with actual released version ([#73](https://github.com/joshrotenberg/mdbook-lint/pull/73)) ([e824b10](https://github.com/joshrotenberg/mdbook-lint/commit/e824b10ea705d101afb038d4e205c6233bfef132))
- Implement rule override mechanism for MDBOOK025/MD025 ([#70](https://github.com/joshrotenberg/mdbook-lint/pull/70)) ([b206da8](https://github.com/joshrotenberg/mdbook-lint/commit/b206da8b423b2e1fa4e1df21d57d81d4640ef810))
- Update CLI crate dependency to mdbook-lint-core 0.4.0 ([#68](https://github.com/joshrotenberg/mdbook-lint/pull/68)) ([fffdf63](https://github.com/joshrotenberg/mdbook-lint/commit/fffdf63c18a4270712f600f4f810d509397e5e7d))


### Documentation
- Improve project description in README ([#76](https://github.com/joshrotenberg/mdbook-lint/pull/76)) ([d7197e1](https://github.com/joshrotenberg/mdbook-lint/commit/d7197e12cf80422e717d8b281ac322cf4ebf69c1))


### Features
- Add release-please configuration for Rust workspace ([#72](https://github.com/joshrotenberg/mdbook-lint/pull/72)) ([63a4dbe](https://github.com/joshrotenberg/mdbook-lint/commit/63a4dbe749616de5035e5e405bf7e2f4c1032b59))


### Miscellaneous
- Sync release-please manifest to current version 0.4.1 ([#71](https://github.com/joshrotenberg/mdbook-lint/pull/71)) ([b7b6cc2](https://github.com/joshrotenberg/mdbook-lint/commit/b7b6cc24d344979dd140ac310731033171e9cfe3))
- Bump version to 0.4.1 ([#69](https://github.com/joshrotenberg/mdbook-lint/pull/69)) ([4df30d7](https://github.com/joshrotenberg/mdbook-lint/commit/4df30d7e312108bc53e03f27752243bb77913b44))

## [0.4.0] - 2025-08-07

### Bug Fixes
- MD030 false positives in code blocks and add LSP config loading ([#67](https://github.com/joshrotenberg/mdbook-lint/pull/67)) ([f813f00](https://github.com/joshrotenberg/mdbook-lint/commit/f813f00449731c06573de3fe3f2a955f1051b095))
- Exclude SUMMARY.md from MD025 multiple H1 check ([#65](https://github.com/joshrotenberg/mdbook-lint/pull/65)) ([8b3b702](https://github.com/joshrotenberg/mdbook-lint/commit/8b3b702780002e44489f8a193ac8961c0d0fd41f))
- Resolve corpus testing markdownlint integration ([#49](https://github.com/joshrotenberg/mdbook-lint/pull/49)) ([bc13242](https://github.com/joshrotenberg/mdbook-lint/commit/bc13242a62809841020371b63d86ef6dfb3e0728))


### Features
- Implement MDBOOK007 rule for include file validation ([#60](https://github.com/joshrotenberg/mdbook-lint/pull/60)) ([dfb97d8](https://github.com/joshrotenberg/mdbook-lint/commit/dfb97d848448647b979e20a9096173e31ce8b847))
- Implement MDBOOK006 rule for internal cross-reference validation ([#59](https://github.com/joshrotenberg/mdbook-lint/pull/59)) ([a35c640](https://github.com/joshrotenberg/mdbook-lint/commit/a35c640dee9794bc675e406db696e27bad7ecad9))
- Implement MDBOOK005 rule for orphaned file detection ([#58](https://github.com/joshrotenberg/mdbook-lint/pull/58)) ([7bd7582](https://github.com/joshrotenberg/mdbook-lint/commit/7bd75826bf2a8014d42516c6448db125701919ef))
- Comprehensive corpus testing infrastructure overhaul ([#54](https://github.com/joshrotenberg/mdbook-lint/pull/54)) ([cc0d0cc](https://github.com/joshrotenberg/mdbook-lint/commit/cc0d0cc69d48628dbd9394e7016a5745211fe6df))


### Miscellaneous
- *(main)* Release 0.4.0 ([#52](https://github.com/joshrotenberg/mdbook-lint/pull/52)) ([949c58b](https://github.com/joshrotenberg/mdbook-lint/commit/949c58b03359ca3be47448141298c9088d5bb7b5))


### Refactoring
- Convert to workspace architecture for LSP preparation ([#61](https://github.com/joshrotenberg/mdbook-lint/pull/61)) ([c098171](https://github.com/joshrotenberg/mdbook-lint/commit/c098171f5e66afac561c736c7063f6e2d638d491))

## [0.3.0] - 2025-08-05

### Features
- Add prebuilt binaries for multiple platforms ([#47](https://github.com/joshrotenberg/mdbook-lint/pull/47)) ([8933513](https://github.com/joshrotenberg/mdbook-lint/commit/89335131fcfac4d2bfcdf458814b70aa3c7b9c8f))


### Miscellaneous
- *(main)* Release 0.3.0 ([#48](https://github.com/joshrotenberg/mdbook-lint/pull/48)) ([8717444](https://github.com/joshrotenberg/mdbook-lint/commit/8717444fa0d0fcd5fea437e9d20c8557cbeb055c))

## [0.2.0] - 2025-08-05

### Bug Fixes
- *(md044)* Resolve Unicode panic with emoji characters ([#41](https://github.com/joshrotenberg/mdbook-lint/pull/41)) ([192ad79](https://github.com/joshrotenberg/mdbook-lint/commit/192ad796ae577b1a716783513b5cdf8bb1a01748))
- *(rules)* Eliminate duplicate violations between MDBOOK and standard MD rules ([#38](https://github.com/joshrotenberg/mdbook-lint/pull/38)) ([b04b74f](https://github.com/joshrotenberg/mdbook-lint/commit/b04b74f74ff8c5ee495d8a0a3801c3de214f163b))
- *(rules)* Prevent MD044 from flagging proper names in URL contexts ([#37](https://github.com/joshrotenberg/mdbook-lint/pull/37)) ([8276772](https://github.com/joshrotenberg/mdbook-lint/commit/8276772fcbfdc8615adf433fcd919518bc001d87))
- *(rules)* Prevent MD030 from flagging bold text as list markers ([#35](https://github.com/joshrotenberg/mdbook-lint/pull/35)) ([83bf032](https://github.com/joshrotenberg/mdbook-lint/commit/83bf032f79601fa9c4f4eb1ba72ceeb2bf8c3ab5))


### Features
- Add markdownlint compatibility mode ([#40](https://github.com/joshrotenberg/mdbook-lint/pull/40)) ([4f99765](https://github.com/joshrotenberg/mdbook-lint/commit/4f9976538cb275fc768e1f4355123174048a4874))


### Miscellaneous
- *(main)* Release 0.2.0 ([#36](https://github.com/joshrotenberg/mdbook-lint/pull/36)) ([a1240b5](https://github.com/joshrotenberg/mdbook-lint/commit/a1240b529dad2901c4dc6dd94ea1515624034896))


### Testing
- Reduce test corpus size from 98MB to 12MB ([#33](https://github.com/joshrotenberg/mdbook-lint/pull/33)) ([bd56b62](https://github.com/joshrotenberg/mdbook-lint/commit/bd56b62f50f90335b76d4ff438a394c4baf84955))


### Optimize
- *(ci)* Improve CI/CD pipeline performance ([#42](https://github.com/joshrotenberg/mdbook-lint/pull/42)) ([b1bc2f8](https://github.com/joshrotenberg/mdbook-lint/commit/b1bc2f8ad26cd1a81f08c06d7f4761edce5b531c))

## [0.1.0] - 2025-08-04

### Bug Fixes
- *(ci)* Resolve GitHub Pages deployment configuration ([#6](https://github.com/joshrotenberg/mdbook-lint/pull/6)) ([c9a1c0a](https://github.com/joshrotenberg/mdbook-lint/commit/c9a1c0ae7ff30729e186a6d23c5c4071c8d7ad28))
- *(ci)* Release please permissions ([#4](https://github.com/joshrotenberg/mdbook-lint/pull/4)) ([fa86ff9](https://github.com/joshrotenberg/mdbook-lint/commit/fa86ff950a31e81765ca51beaf9280c0427d3f91))
- *(ci)* Simplify release-please configuration ([#2](https://github.com/joshrotenberg/mdbook-lint/pull/2)) ([4b65b76](https://github.com/joshrotenberg/mdbook-lint/commit/4b65b766a45ceb752182cff5445572fd2d9e0d89))
- *(ci)* Resolve CI failures and establish project governance ([#1](https://github.com/joshrotenberg/mdbook-lint/pull/1)) ([a81a990](https://github.com/joshrotenberg/mdbook-lint/commit/a81a990ec68b1cf6b92895ff3ec2fc1c4d9fd0a5))


### Documentation
- Consolidate contributing documentation into single comprehensive guide ([#9](https://github.com/joshrotenberg/mdbook-lint/pull/9)) ([d2ff9ac](https://github.com/joshrotenberg/mdbook-lint/commit/d2ff9accdc017eb04ef025a308f79704ac68577a))
- Add basic documentation site with GitHub Pages ([#5](https://github.com/joshrotenberg/mdbook-lint/pull/5)) ([e51f03d](https://github.com/joshrotenberg/mdbook-lint/commit/e51f03d99399cdedfe23153476c71a288c0f8982))


### Features
- Improve test coverage with comprehensive edge case testing ([#7](https://github.com/joshrotenberg/mdbook-lint/pull/7)) ([5ffe072](https://github.com/joshrotenberg/mdbook-lint/commit/5ffe07296703d185b2516652aa2d2fe00340a901))


### Miscellaneous
- *(main)* Release 0.1.0 ([#3](https://github.com/joshrotenberg/mdbook-lint/pull/3)) ([a78f99a](https://github.com/joshrotenberg/mdbook-lint/commit/a78f99ad93f1e27ef47856b7d02ee7903b579283))


