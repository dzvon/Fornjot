use std::slice;

use fj_math::{Scalar, Winding};
use itertools::Itertools;

use crate::{geometry::curve::Curve, objects::HalfEdge, storage::Handle};

/// A cycle of connected half-edges
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Cycle {
    half_edges: Vec<Handle<HalfEdge>>,
}

impl Cycle {
    /// Create an instance of `Cycle`
    pub fn new(half_edges: impl IntoIterator<Item = Handle<HalfEdge>>) -> Self {
        let half_edges = half_edges.into_iter().collect::<Vec<_>>();
        Self { half_edges }
    }

    /// Access the half-edges that make up the cycle
    pub fn half_edges(&self) -> HalfEdgesOfCycle {
        self.half_edges.iter()
    }

    /// Indicate the cycle's winding, assuming a right-handed coordinate system
    ///
    /// Please note that this is not *the* winding of the cycle, only one of the
    /// two possible windings, depending on the direction you look at the
    /// surface that the cycle is defined on from.
    pub fn winding(&self) -> Winding {
        // The cycle could be made up of one or two circles. If that is the
        // case, the winding of the cycle is determined by the winding of the
        // first circle.
        if self.half_edges.len() < 3 {
            let first = self
                .half_edges()
                .next()
                .expect("Invalid cycle: expected at least one half-edge");

            let [a, b] = first.boundary();
            let edge_direction_positive = a < b;

            let circle = match first.curve() {
                Curve::Circle(circle) => circle,
                Curve::Line(_) => unreachable!(
                    "Invalid cycle: less than 3 edges, but not all are circles"
                ),
            };
            let cross_positive = circle.a().cross2d(&circle.b()) > Scalar::ZERO;

            if edge_direction_positive == cross_positive {
                return Winding::Ccw;
            } else {
                return Winding::Cw;
            }
        }

        // Now that we got the special case out of the way, we can treat the
        // cycle as a polygon:
        // https://stackoverflow.com/a/1165943

        let mut sum = Scalar::ZERO;

        for (a, b) in self.half_edges.iter().circular_tuple_windows() {
            let [a, b] = [a, b].map(|half_edge| half_edge.start_position());

            sum += (b.u - a.u) * (b.v + a.v);
        }

        if sum > Scalar::ZERO {
            return Winding::Cw;
        }
        if sum < Scalar::ZERO {
            return Winding::Ccw;
        }

        unreachable!("Encountered invalid cycle: {self:#?}");
    }
}

/// An iterator over the half-edges of a [`Cycle`]
///
/// Returned by [`Cycle::half_edges`].
pub type HalfEdgesOfCycle<'a> = slice::Iter<'a, Handle<HalfEdge>>;
