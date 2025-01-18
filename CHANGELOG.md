# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.2](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v1.3.1...sylvia-derive-v1.3.2) - 2024-12-09

### Other

- Remove derivative dependency and fix lints

## [1.3.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v1.2.1...sylvia-derive-v1.3.0) - 2024-11-20

### Added

- Assert no attributes used on `self` and `ctx`
- Error on wrong `sv::data` and `sv::payload` usage
- Return transaction id and block height if deserialization failed
- Assert no redundant parameters in reply method
- Update MultiTest to 2.2.0
- Revert adding Error to ContractApi
- Create ctx module for context types
- Add temporary `sylvia::replies::ReplyCtx` with additional fields ([#450](https://github.com/CosmWasm/sylvia/pull/450))
- Support omitting data parameter
- Add `instantiate` parameter to the `sv::data` attribute
- Add auto deserialization of reply data ([#445](https://github.com/CosmWasm/sylvia/pull/445))
- Add support for `sv::features` attribute. ([#446](https://github.com/CosmWasm/sylvia/pull/446))
- Pass payload while building SubMsg ([#441](https://github.com/CosmWasm/sylvia/pull/441))
- Add option to deserialize payload ([#439](https://github.com/CosmWasm/sylvia/pull/439))
- Impl SubMsg trait on CosmosMsg
- Add InstantiateBuilder and SubMsg trait
- Emit new reply dispatch ([#429](https://github.com/CosmWasm/sylvia/pull/429))
- Generate unique reply_ids ([#427](https://github.com/CosmWasm/sylvia/pull/427))
- Parse additional parameters for sv::msg(reply) ([#426](https://github.com/CosmWasm/sylvia/pull/426))
- Make serde dep user side obsolete
- Make cosmwasm_schema deps obsolete
- Make user side cosmwasm-std dep obsolete

### Other

- Add description of the `data` and `payload` attributes.
- Better error message for `sv::features` attribute
- Store MsgVariant in ReplyData
- Improve reply generation
- Remove `msg_type` method from `MsgVariant`
- Store `MsgAttr` in `MsgVariant`
- Make `MsgAttr` a struct

## [1.2.1](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v1.2.0...sylvia-derive-v1.2.1) - 2024-08-22

### Fixed

- Allow deprecated InterfaceApi in generated code
- Revert removal of `, custom` parameter support in entry_points macro
- Revert removal of InterfaceApi
- Deprecate `, custom` parameter in `entry_points` macro. Will be removed in `2.0.0`.
- Deprecate `InterfaceApi`. Will be removed in `2.0.0`. Use generated `sv::InterfaceMessagesApi` instead.

## [1.2.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v1.1.0...sylvia-derive-v1.2.0) - 2024-08-20

### Added

- Emit error if contract macro is above entry_points
- Remove support for `, custom(msg=.. ,query=..)` in entry_points
- Add CustomMsg and CustomQuery to ContractApi

### Fixed

- Pass attribute to struct fields

### Other

- *(sylvia-derive)* Document inner types
- [**breaking**] Remove `InterfaceApi` in favor of `InterfaceMessagesApi` ([#413](https://github.com/CosmWasm/sylvia/pull/413))
- Fix duplicated instantiation error
- Refactor struct message generation
- Generic cw1-whitelist example ([#404](https://github.com/CosmWasm/sylvia/pull/404))
- Create fold module
- Setup new directory structure
- Update to cosmwasm-std 2.1.1
- Cleanup in entry_points macro
- Silence clippy warn about lack of Default ([#396](https://github.com/CosmWasm/sylvia/pull/396))

## [1.1.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v1.0.2...sylvia-derive-v1.1.0) - 2024-07-12

### Added

- `From` implementation for contract messages of interface messages ([#391](https://github.com/CosmWasm/sylvia/pull/391))
- Attributes forwarding to message enums and fields ([#388](https://github.com/CosmWasm/sylvia/pull/388))
- Executors ([#386](https://github.com/CosmWasm/sylvia/pull/386))
- No ref needed for 'dyn Interface' type in Remote and BoundQuerier ([#382](https://github.com/CosmWasm/sylvia/pull/382))
- Remove types forwarding to interface in sv::messages ([#361](https://github.com/CosmWasm/sylvia/pull/361))

### Other

- Update documentation and refactoring ([#393](https://github.com/CosmWasm/sylvia/pull/393))
- Add trybuild check for two instantiate methods ([#392](https://github.com/CosmWasm/sylvia/pull/392))
- Update dependecies
- Update proc-macro-crate deps

## [1.0.2](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v1.0.1...sylvia-derive-v1.0.2) - 2024-04-24

### Other

- Update `cw_multi_test`
- Split big chunks of code in `sylvia_derive::multitest`
- Provide `to_case` functionality to `syn::Ident`
- Remove `stripped_return_type` from `MsgVariant`
- Remove duplicated `querier` related code from `MsgVariant`
- Move `Multitest` related `MsgVariant` logic to trait in multitest module
- Unify MT proxy methods emit

## [1.0.1](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v1.0.0...sylvia-derive-v1.0.1) - 2024-04-15

### Added

- Assert `new` method defined ([#342](https://github.com/CosmWasm/sylvia/pull/342))

### Fixed

- Add missing `map_err` on `IntoResponse` result

### Other

- Improve error message in `sv::custom` attribute ([#348](https://github.com/CosmWasm/sylvia/pull/348))
- Improve errors in `sv::messages` attribute ([#345](https://github.com/CosmWasm/sylvia/pull/345))
- Improve `no instantiation` error message

## [1.0.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.10.1...sylvia-derive-v1.0.0) - 2024-03-27

### Added

- Update deps to 2.0.0 ([#308](https://github.com/CosmWasm/sylvia/pull/308))

## [0.10.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.9.3...sylvia-derive-v0.10.0) - 2024-03-26

### Added

- Implement Querier on App ([#154](https://github.com/CosmWasm/sylvia/pull/154))
- Change multitest modules names to unified `mt` ([#324](https://github.com/CosmWasm/sylvia/pull/324))
- Handle missing explicite custom types ([#323](https://github.com/CosmWasm/sylvia/pull/323))
- BoundQuerier improve ([#321](https://github.com/CosmWasm/sylvia/pull/321))
- Remove `#[contract(module=...)]` support ([#320](https://github.com/CosmWasm/sylvia/pull/320))
- `#[contract(module=...)]` not needed in mt ([#319](https://github.com/CosmWasm/sylvia/pull/319))
- `#[contract(module=...)]` and `#[messages]` not needed for trait impl ([#318](https://github.com/CosmWasm/sylvia/pull/318))
- Remove custom in impl trait ([#314](https://github.com/CosmWasm/sylvia/pull/314))
- Error on missing module for `impl Interface for Contract` ([#311](https://github.com/CosmWasm/sylvia/pull/311))
- Add support for `#[sv::]` attributes for all sylvia attribtues. ([#310](https://github.com/CosmWasm/sylvia/pull/310))
- Generate sudo multitest helpers
- Generate sudo entry point
- Generate SudoMsg in contract
- Generate SudoMsg in interface
- Forward generics through associated types
- ExecC and QueryC used in place of CustomMsgT
- Remove mt trait_utils generation
- Impl interfaces with associated types on generic contract
- Impl interface with associated types on non generic contract
- Interfaces generate with associated_types
- Allow specifying concrete customs in entry_points
- Support duplicated exec generic params
- Forward generics to custom_generic interface
- Allow single concrete type to be passed in place of multiple
- Forward generics to the interface
- Make as Variant optional for #[messages(...)] attribute

### Other

- Update README ([#331](https://github.com/CosmWasm/sylvia/pull/331))
- Enable all features in docs.rs and add multitest docs comments
- Enable code examples in macros doc tests
- Remove tarpaulin exclusions ([#312](https://github.com/CosmWasm/sylvia/pull/312))
- Add sudo to custom example
- Update README.md ([#300](https://github.com/CosmWasm/sylvia/pull/300))
- Change function signature
- Update docs ([#299](https://github.com/CosmWasm/sylvia/pull/299))
- Internal renaming
- Create ImplMtHelpers
- Impl non-generic non-custom on forwarding contract
- Migrate to syn 2.0

## [0.9.2](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.9.1...sylvia-derive-v0.9.2) - 2023-11-29

### Added

- Allow querying code_info from CodeId and App
- Add cosmwasm_1_2 feature flag
- Allow specifying salt for contract address

## [0.9.1](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.9.0...sylvia-derive-v0.9.1) - 2023-11-17

### Other

- Update deps

## [0.9.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.8.1...sylvia-derive-v0.9.0) - 2023-11-13

### Added

- Support generic types in entry points
- Impl ContractApi extension trait
- Wrap contract types in `sv` module
- Wrap impl types in `sv` module
- Hide generated interface types in `sv` module
- Move `messages` method out of EnumMsg
- Support generics on every message type
- Support generic contract for simple contract
- Support generic contract for simple contract
- Support generics on `messages` attribute in main `contract` macro
- Support generic interface implemented on contract
- Emit InterfaceTypes
- Check interfaces return type for used generics
- Add support for generics in interface

### Fixed

- [**breaking**] Expect `,` in `:custom(msg, query)`

## [0.8.1](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.8.0...sylvia-derive-v0.8.1) - 2023-09-18

### Added

- Generate migrate entry point if message defined on contract

## [0.8.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.7.1...sylvia-derive-v0.8.0) - 2023-09-05

### Added

- Cast `deps` to empty
- Support QueryC associated type on interface
- Support custom queries on contracts

## [0.7.1](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.7.0...sylvia-derive-v0.7.1) - 2023-08-14

### Fixed

- Prefix interface proxy with module as Path

## [0.7.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.6.1...sylvia-derive-v0.7.0) - 2023-08-01

### Added

- Override generated entry_points
- Override entry_points in multitest helpers

### Fixed

- [**breaking**] `Remote` type implements all relevant traits so it can be stored in `#[cw_serde]` types

## [0.6.1] - 2023-06-28

- Fix dependencies in sylvia 0.6.0 (0.6.0 will be yanked)

## [0.6.0] - 2023-06-28

- InstantiateCtx and ReplyCtx are no longer type aliases (breaking)
- `multitest::App` is using more generic multitest version of `App`
- Support for custom messages via `#[sv::custom]` attribute

## [0.5.0] - 2023-05-26

- New `BoundQuerier` and `Remote` types are generated. Their goal is to make
  querying other contracts more intuitive.
- `module` attr for `contract` macro no longer wraps generated code in scope.
  As from now it will be used to provide path to contract implementation.
- Removed requirement for `const fn new()` method for `contract` macro call.
  `fn new()` method is still required.

## [0.4.2] - 2023-05-24

- Added support of `#[sv::msg(reply)]` defining handler for reply messages,
  currently only in the form of
  `fn reply(&self, _ctx: ReplyCtx, _msg: Reply) -> Result<Response, Err>`
- Added generation of reply entrypoint forwarding to the `#[sv::msg(reply)]`
  handler
- Added generation of reply implementation forwarding to `#[sv::msg(reply)]`
  handler in multitest helpers

## [0.4.1] - 2023-05-23

- Lint fix

## [0.4.0] - 2023-05-16

- Introduced new `entry_points` macro
- Custom errors can be passed through `error` attribute

## [0.3.2] - 2023-04-18

- Changed the way multitest helpers are generated to avoid weird `use` statements in code.
- Introduced Context types in place of tuples
- Forwarding attributes on message fields
- Example usage of generated multitest helpers

## [0.3.1] - 2023-03-03

- Slight improvement the invalid message received error

## [0.3.0] - 2023-02-01

- Interfaces moved to separate directory to avoid errors on workspace optimizer
- `mt` feature added. Enabling it will:
  - generate `cw_multi_test::Contract` impl on a contract
  - generate Proxy to simplify writting tests
- Example of usage of new test framework
- Port of `cw20` contract on `sylvia` framework
- Default error type on contract is now `cosmwasm_std::StdError`
- Reexported `schemars`

## [0.2.2] - 2022-12-13

- Fix: Generate Migrate as struct
- Cw20 implementation in sylvia
- Removed `#[sv::msg(reply)]`

## [0.2.1] - 2022-10-19

This is the first documented and supported implementation. It provides
macro to generate messsages for interfaces and contracts.

Some main points:

- Support for instantiate, execute, query, migrate and reply messages.
- Ability to implement multiple interfaces on contract.
- Mechanism of detecting overlapping of messages.
- Dispatch mechanism simplyfing entry points creation.
- Support for schema generation.
