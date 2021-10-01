#[cfg(feature = "default")]
mod definitions;

mod error;
mod extrinsics;
mod modules;
mod overrides;
mod resolver;
pub mod runtimes;

pub use self::error::*;
pub use self::extrinsics::*;
pub use self::modules::*;
pub use self::overrides::*;
pub use self::resolver::{Builder as TypeResolverBuilder, TypeResolver};

use serde::{Deserialize, Serialize};
/// An overrides for a range of runtime versions
#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Clone)]
pub struct TypeRange {
	/// the spec these overrides are relevant for
	#[serde(rename = "minmax")]
	min_max: [Option<usize>; 2],
	/// types that are being overriden
	/// points to the types that should be used instead in definitions.json
	types: ModuleTypes,
}

/// Check if a type is in the range of `TypeRange`
fn is_in_range(spec: u32, over_ride: &TypeRange) -> bool {
	match over_ride.min_max {
		[Some(min), Some(max)] => (min..=max).contains(&(spec as usize)),
		[Some(min), None] => (spec as usize) >= min,
		[None, Some(max)] => (spec as usize) <= max,
		// presumably, this would be for null -> null,
		// so for every spec
		[None, None] => true,
	}
}
