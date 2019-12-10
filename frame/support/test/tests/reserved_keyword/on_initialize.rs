macro_rules! reserved {
	($($reserved:ident)*) => {
		$(
			mod $reserved {
				use support::additional_traits::MaybeDoughnutRef;
				pub use support::dispatch::Result;

				// `decl_module` expansion has added doughnut logic which requires system trait is implemented
				pub trait Trait: system::Trait {
					type Origin: MaybeDoughnutRef<Doughnut=()>;
					type BlockNumber: Into<u32>;
				}

				pub mod system {
					use sp_runtime::traits::DoughnutApi;
					use support::additional_traits::DelegatedDispatchVerifier;
					use support::dispatch::Result;

					pub trait Trait {
						type Doughnut: DoughnutApi;
						type DelegatedDispatchVerifier: DelegatedDispatchVerifier<()>;
					}

					pub fn ensure_root<R>(_: R) -> Result {
						Ok(())
					}
				}

				support::decl_module! {
					pub struct Module<T: Trait> for enum Call where origin: T::Origin {
						fn $reserved(_origin) -> Result { unreachable!() }
					}
				}
			}
		)*
	}
}

reserved!(on_finalize on_initialize on_finalise on_initialise offchain_worker deposit_event);

fn main() {}
