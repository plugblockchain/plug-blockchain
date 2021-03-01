# Implementing PoA for a Pl^g blockchain

Proof of Authority requires a set of managed authorities who can be added and removed from the chain by root.

The `pallet-validator-manager` allows us to manage who can paritcilpate in block production and finalization.

PoA is tightly coupled with block production; so we will need to integrate `validator-manager` with:
* `Aura` 
    * `Aura` selects which nodes are allowed to author blocks 
* `Grandpa`
    * `Grandpa` is a finality gadget which allows all `Validators` to handle forks
* `Session`
    * `Session` allows users to map thier `AccountId` to specific nodes that they operate

## 1. Adding ValidatorManager to the Runtime

### 1.1. Add prml_validator_manager to the runtime crate

In the runtime `Cargo.toml`:
```
[dependencies]
prml-validator-manager = { git = "https://github.com/plugblockchain/plug-blockchain", default-features = false }
...
[features]
default = ["std"]
std = [
	...
	"prml-validator-manager/std",
]
```

### 1.2. Add the Validator Manager to the Runtime Enum

In `src/lib.rs` under the `Runtime` enum:
```rust
ValidatorManager: prml_validator_manager::{Module, Call, Storage, Config<T>, Event<T>},
```

### 1.3 Implement the correct Session Keys for PoA

At a minimum, your `SessionKeys` should have `Aura` and `Grandpa` in `src/lib.rs`:
```rust
pub mod opaque {
	//...
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}
```

### 1.3. Implement PoA Traits for Runtime

In `src/lib.rs`:

```rust
parameter_types! {
    // Set the minimum number of validators here - root will not be able to remove 
    // validators if the number of validators are less or equal to this number
	pub const MinimumValidatorCount: u32 = 1;
}

impl prml_validator_manager::Trait for Runtime {
	type Event = Event;
	type MinimumValidatorCount = MinimumValidatorCount;
}

impl pallet_aura::Trait for Runtime {
	type AuthorityId = AuraId;
}

impl pallet_grandpa::Trait for Runtime {
	type Event = Event;
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
	pub const Period: BlockNumber = 10;
	pub const Offset: BlockNumber = 0;
}

impl pallet_session::Trait for Runtime {
	type SessionManager = ValidatorManager;
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type Event = Event;
	type Keys = opaque::SessionKeys;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = ConvertInto;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}
```

In the session trait implementation:
* We are using `ValidatorManager` to manage new validator sets
* We are using `PeriodicSessions` to make sessions last 10 blocks
* The `SessionManager` interfaces with `Aura` and `Grandpa` through the `SessionKeys`

## 2. Configure the TestNet Genesis Config

Genesis Config needs a set of authorities to author and validate blocks.

### 2.1. Import Validator Manager Config

```rust
use plug_runtime::ValidatorManagerConfig;
```

Add your set of validators under the `GenesisConfig` struct:
```rust
    prml_validator_manager: Some(ValidatorManagerConfig {
		validators: initial_authority_account_ids.clone(),
    }),
```
