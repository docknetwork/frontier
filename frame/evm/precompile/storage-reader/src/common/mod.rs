pub mod output;
pub mod params;

/// Ensures that caller has enough gas, producing `ExitError::OutOfGas` in case of failure.
#[macro_export]
macro_rules! ensure_enough_gas {
	($target_gas: ident >= $gas_required: ident) => {
		if let Some(target_gas) = $target_gas {
			if target_gas < $gas_required {
				return Err(evm::ExitError::OutOfGas.into());
			}
		}
	};
}
