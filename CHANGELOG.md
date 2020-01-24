# Changelog
Track changes made between Plug and it's upstream project `Substrate`.
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Added
- Add `MultiCurrencyAccounting` trait to support multi currency accounting in contracts module (#39)

`frame/contracts/src/exec.rs`
- Add optional doughnut type to `ExecutionContext` struct (#44)
- Add `Ext::doughnut()` to get an optional doughnut from a contract (#44)

`frame/system/src/lib.rs`
- Add `fn ensure_verified_contract_call()` to verify a doughnut attached to origin at `Contract::call()` (#47)

`frame/support/src/additional_traits.rs`
- Add `DummyDispatchVerifier` struct for easier mock up in tests (#47)

`frame/contracts/src/exec.rs`
- Add `DelegatedRuntimeCall` variant to `DeferredAction` enum (#48)
- Add `fn note_delegated_dispatch_call()` to dispatch a delegated contract call (#48)

`frame/contracts/src/wasm/runtime.rs`
- Add `ext_delegated_dispatch_call` method to dispatch a delegated contract cal (#48)

## Changed
- Renamed trait `AssetIdProvider` to `AssetIdAuthority` to reflect it's 'read from chain' behaviour (#39)
- Make GA imbalance types currency aware so that issuance is managed properly on Drop (#39)

`frame/contracts/src/wasm/runtime.rs`
- Change `ext_call` method to verify doughnut if attached to a contract (#44)

------

## [1.0.0-rc1] - 2019-10-21

### Added

`prml/attestation/*`
- Added attestation runtime module

`prml/doughnut/*`
- Add `PlugDoughnut` wrapper struct to allow doughnut integration with `SignedExtension` hooks
- Add `PlugDoughnutDispatcher` as the defacto Plug implementor for `DelegatedDispatchVerifier`

`node/runtime/src/lib.rs`
- Add doughnut proof as an optional first parameter to the `node/runtime` `SignedExtra` payload allowing doughnut's to be added to extrinsics
- Add `DelegatedDispatchVerifier` and `Doughnut` proof type to `system::Trait` type bounds

`primitives/runtime/src/traits.rs`
- Blanket impl SignedExtension for `Option<T>` to allow Optional<PlugDoughnut> in extrinsics
- Add `MaybeDoughnut` trait for SignedExtension type to allow extracting doughnut from `SignedExtra` tuple
- impl `MaybeDoughnut` for SignedExtension macro tuple of all lengths

### Changed

`primitives/runtime/src/traits.rs`
- Make trait `SignedExtension::pre_dispatch` method receive self by reference (`&self`), instead of move (`self`)

`frame/staking/*`
- Add `RewardCurrency` type to allow paying out staking rewards in a different currency to the staked currency
- Change `fn make_payout()` so that RewardCurrency is paid to the stash account and not added to the total stake, if the reward currency is not the staked currency

`frame/system/src/lib.rs` and `frame/support/src/origin.rs`
- Add `DelegatedOrigin` variant to `RawOrigin` for delegated transactions
- Add `MaybeDoughnutRef` trait for extracting doughnut proof from `origin` without move in runtime module methods

`frame/support/src/dispatch.rs`
- Add `DelegatedDispatchVerifier` check to `decl_module!` expansion. This allows doughnut proofs to be checked when an extrinsic is dispatched using the `<T as system::Trait>::DelegatedDispatchVerifier` impl
- Renamed binary to `plug` changes made to (`Cargo.toml`s and `Dockerfile` to support this)

### Removed

- The majority of `docs/` is substrate specific and has been removed
- `README.adoc` is substrate specific and has been removed
