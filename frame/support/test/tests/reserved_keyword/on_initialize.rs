macro_rules! reserved {
	($($reserved:ident)*) => {
		$(
			mod $reserved {
				use frame_support::additional_traits::MaybeDoughnutRef;
				pub use frame_support::dispatch;

				// `decl_module` expansion has added doughnut logic which requires system trait is implemented
				pub trait Trait: system::Trait {
					type Origin: MaybeDoughnutRef<Doughnut=()>;
					type BlockNumber: Into<u32>;
				}

				pub mod system {
					use frame_support::dispatch;
					use frame_support::additional_traits::DelegatedDispatchVerifier;
					use sp_runtime::traits::PlugDoughnutApi;

					pub trait Trait {
						type Doughnut: PlugDoughnutApi;
						type DelegatedDispatchVerifier: DelegatedDispatchVerifier<Doughnut = ()>;
					}

					pub fn ensure_root<R>(_: R) -> dispatch::DispatchResult {
						Ok(())
					}
				}

				frame_support::decl_module! {
					pub struct Module<T: Trait> for enum Call where origin: T::Origin {
						fn $reserved(_origin) -> dispatch::DispatchResult { unreachable!() }
					}
				}
			}
		)*
	}
}

reserved!(on_finalize on_initialize on_finalise on_initialise offchain_worker deposit_event);

fn main() {}
