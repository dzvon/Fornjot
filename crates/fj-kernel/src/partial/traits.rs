use std::fmt::Debug;

use crate::{objects::Objects, services::Service};

use super::FullToPartialCache;

/// Implemented for objects that a partial object variant exists for
pub trait HasPartial {
    /// The type representing the partial variant of this object
    type Partial: PartialObject<Full = Self>;
}

/// Implemented for partial objects
pub trait PartialObject: Clone + Debug {
    /// The type representing the full object
    type Full: HasPartial<Partial = Self>;

    /// Construct a default partial object
    fn new(objects: &mut Service<Objects>) -> Self;

    /// Construct a partial object from a full one
    fn from_full(full: &Self::Full, cache: &mut FullToPartialCache) -> Self;

    /// Build a full object from the partial object
    fn build(self, objects: &mut Service<Objects>) -> Self::Full;
}

macro_rules! impl_trait {
    ($($full:ident, $partial:ident;)*) => {
        $(
            impl HasPartial for crate::objects::$full {
                type Partial = super::$partial;
            }
        )*
    };
}

impl_trait!(
    Face, PartialFace;
    Shell, PartialShell;
    Sketch, PartialSketch;
    Solid, PartialSolid;
);
