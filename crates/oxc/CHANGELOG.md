# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).



## [0.77.2] - 2025-07-17

### 🚜 Refactor

- eb12132 minifier: Unify access `CompressOptions` through `ctx.state` (#12346) (Boshen)











# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.70.0] - 2025-05-15

### Testing

- a05361e ast/estree: Check span offsets are converted in ascending order in ESTree conformance tests (#10887) (overlookmotel)

## [0.68.0] - 2025-05-03

- 315143a codegen: [**BREAKING**] Remove useless `CodeGenerator` type alias (#10702) (Boshen)

### Bug Fixes

- 4795059 transformer_plugins: Provide reference data when identifiers are replaced (#10620) (Boshen)

### Refactor


## [0.67.0] - 2025-04-27

### Features

- 1962bc6 transformer_plugins: Split out `oxc_transformer_plugins` crate (#10617) (Boshen)

### Testing

- fbd6864 transformer_plugins: Apply DCE after replace plugin (#10619) (Boshen)

## [0.61.0] - 2025-03-20

### Features

- a9a47a6 parser: Add regex cargo feature to oxc_parser (#9879) (Toshit)

## [0.57.0] - 2025-03-11

- ef6e0cc semantic: [**BREAKING**] Combine `SymbolTable` and `ScopeTree` into `Scoping` (#9615) (Boshen)

- 23738bf semantic: [**BREAKING**] Introduce `Scoping` (#9611) (Boshen)

### Refactor


## [0.55.0] - 2025-03-05

### Features

- d55dbe2 ast/estree: Raw transfer (experimental) (#9516) (overlookmotel)

## [0.54.0] - 2025-03-04

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

### Features


## [0.49.0] - 2025-02-10

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.48.2] - 2025-02-02

### Refactor

- 6aa2dde codegen: Accept SymbolTable instead of Mangler (#8829) (Daniel Bulant)

## [0.47.0] - 2025-01-18

- 4ce6329 semantic: [**BREAKING**] Ensure program outlives semantic (#8455) (Valentinas Janeiko)

### Bug Fixes


## [0.42.0] - 2024-12-18

### Refactor

- 1314c97 minifier: Expose dce as an API instead of an option (#7957) (Boshen)

## [0.39.0] - 2024-12-04

- 8a788b8 parser: [**BREAKING**] Build `ModuleRecord` directly in parser (#7546) (Boshen)

### Features

- 40792b4 napi/parser: Change parse API to accept mandatory `filename` and optional `lang` (#7605) (Boshen)
- 7c62a33 napi/parser: Return esm info (#7602) (Boshen)
- 5864352 napi/transform: Add `TransformerOptions::assumptions` (#7601) (翠 / green)
- 771c698 oxc: Remove `oxc_napi` crate (#7634) (Boshen)
- bd977cf oxc: Add `oxc_napi` crate (#7612) (Boshen)

## [0.38.0] - 2024-11-26

- 5d65656 oxc_index: [**BREAKING**] Move to own repo github.com/oxc-project/oxc-index-vec (#7464) (Boshen)

- bb2c0c2 transformer: [**BREAKING**] Return `String` as error instead of OxcDiagnostic (#7424) (Boshen)

### Features

- 59e7e46 napi/transform: Add `TransformOptions::target` API (#7426) (Boshen)

### Refactor


## [0.36.0] - 2024-11-09

- 846711c transformer: [**BREAKING**] Change API to take a `&TransformOptions` instead of `TransformOptions` (#7213) (Boshen)

### Features


## [0.35.0] - 2024-11-04

### Features

- fcaba4a transformer: Add `TransformerOptions::env` with `EnvOptions` (#7037) (Boshen)

### Bug Fixes

- d15e408 napi/transform: Fix 'typescript.declaration' option not working (#7012) (Boshen)
- b188b4a transformer: Fix typescript globals being recognized as globals (#7100) (Boshen)

## [0.34.0] - 2024-10-26

- 4618aa2 transformer: [**BREAKING**] Rename `TransformerOptions::react` to `jsx` (#6888) (Boshen)

- 67a7bde napi/parser: [**BREAKING**] Add typings to napi/parser (#6796) (ottomated)

### Features


### Bug Fixes

- 4dc5e51 transformer: Only run typescript plugin for typescript source (#6889) (Boshen)
- b075982 types: Change @oxc/types package name (#6874) (ottomated)

### Refactor


## [0.32.0] - 2024-10-19

- 91c87dd codegen: [**BREAKING**] Remove `Codegen::enableSourceMap` API (#6452) (Boshen)

- 7645e5c codegen: [**BREAKING**] Remove CommentOptions API (#6451) (Boshen)

- 5200960 oxc: [**BREAKING**] Remove passing `Trivias` around (#6446) (Boshen)

### Refactor


## [0.31.0] - 2024-10-08

- 020bb80 codegen: [**BREAKING**] Change to `CodegenReturn::code` and `CodegenReturn::map` (#6310) (Boshen)

- 4f6bc79 transformer: [**BREAKING**] Remove `source_type` param from `Transformer::new` (#6251) (overlookmotel)

### Features

- abd3a9f napi/transform: Perform dce after define plugin (#6312) (Boshen)
- a0ccc26 napi/transform: Add `lang` option to change source type (#6309) (Boshen)
- 2f888ed oxc: Add napi transform options (#6268) (Boshen)
- 8729755 oxc,napi/transform: Napi/transform use oxc compiler pipeline (#6298) (Boshen)

### Refactor

- aa0dbb6 oxc: Add `napi` feature, change napi parser to use `oxc` crate (#6265) (Boshen)

## [0.30.2] - 2024-09-27

### Documentation

- d60ceb4 oxc: Add README.md and crate-level docs (#6035) (DonIsaac)

## [0.30.1] - 2024-09-24

### Documentation

- 18371dd oxc: Include feature-guarded modules in docs.rs (#6012) (DonIsaac)

## [0.30.0] - 2024-09-23

### Features

- 3230ae5 semantic: Add `SemanticBuilder::with_excess_capacity` (#5762) (overlookmotel)

### Documentation

- bacfbb8 oxc: Add submodule documentation (#5984) (DonIsaac)

## [0.28.0] - 2024-09-11

- b060525 semantic: [**BREAKING**] Remove `source_type` argument from `SemanticBuilder::new` (#5553) (Boshen)

### Features

- 2016bae coverage: Add regular expression idempotency test (#5676) (Boshen)

### Refactor


## [0.27.0] - 2024-09-06

### Features

- ed8ab6d oxc: Conditional expose `oxc_cfg` in `oxc` crate (#5524) (IWANABETHATGUY)

## [0.26.0] - 2024-09-03

### Features

- be4642f semantic: Transform checker check child scope IDs (#5410) (overlookmotel)

### Refactor

- 3ae94b8 semantic: Change `build_module_record` to accept &Path instead of PathBuf (Boshen)

## [0.25.0] - 2024-08-23

- ce4d469 codegen: [**BREAKING**] Remove const generic `MINIFY` (#5001) (Boshen)

### Features

- 6800e69 oxc: Add `Compiler` and `CompilerInterface` (#4954) (Boshen)

### Refactor

- cd9cf5e oxc: Remove `remove_whitespace` (Boshen)
- b4407c4 oxc,mangler: `oxc` crate add mangler; mangler use options API (Boshen)
- 4fdf26d transform_conformance: Add driver (#4969) (Boshen)

## [0.21.0] - 2024-07-18

### Features

- 8a190eb oxc: Export `oxc_mangler` (Boshen)

## [0.16.0] - 2024-06-26

### Features

- 4fb90eb oxc: Export isolated-declarations (#3765) (Boshen)

## [0.13.0] - 2024-05-14

### Features

- f6daf0b sourcemap: Add feature "sourcemap_concurrent" (Boshen)

## [0.11.0] - 2024-03-30

### Features
- b199cb8 Add oxc sourcemap crate (#2825) (underfin)

## [0.10.0] - 2024-03-14

### Features
- 697b6b7 Merge features `serde` and `wasm` to `serialize` (#2716) (Boshen)

## [0.6.0] - 2024-02-03

### Features

- cd5026c ast: TypeScript definition for wasm target (#2158) (Nicholas Roberts)

## [0.5.0] - 2024-01-12

### Features

- f1b433b playground: Visualize symbol (#1886) (Dunqing)

### Refactor

- a6717db formatter,linter,codegen: Remove oxc_formatter (#1968) (Boshen)

## [0.4.0] - 2023-12-08

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- 2e2b758 playground: Add transform and minify (#993) (Boshen)

