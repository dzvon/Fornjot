pub mod edges;
pub mod faces;
pub mod shapes;

use parry3d_f64::bounding_volume::AABB;

use crate::math::Point;

use self::{edges::Edges, faces::Faces};

/// Implemented by all shapes
pub trait Shape {
    /// Access the axis-aligned bounding box of a shape
    ///
    /// If a shape is empty, its [`Aabb`]'s `min` and `max` points must be equal
    /// (but are otherwise not specified).
    fn bounding_volume(&self) -> AABB;

    /// Compute triangles to approximate the shape's faces
    ///
    /// The shape defined by the approximated triangles must be fully contained
    /// within the actual shape.
    ///
    /// `tolerance` defines by how far this triangulation is allowed to deviate
    /// from the faces' actual dimensions.
    fn faces(&self, tolerance: f64) -> Faces;

    /// Access the edges of the shape
    fn edges(&self) -> Edges;

    /// Return the shape's vertices
    fn vertices(&self) -> Vec<Point>;
}

macro_rules! dispatch {
    ($($method:ident($($arg_name:ident: $arg_ty:ident)*) -> $ret:ty;)*) => {
        impl Shape for fj::Shape {
            $(
                fn $method(&self, $($arg_name: $arg_ty)*) -> $ret {
                    match self {
                        Self::Shape2d(shape) => shape.$method($($arg_name)*),
                        Self::Shape3d(shape) => shape.$method($($arg_name)*),
                    }
                }
            )*
        }

        impl Shape for fj::Shape2d {
            $(
                fn $method(&self, $($arg_name: $arg_ty)*) -> $ret {
                    match self {
                        Self::Circle(shape) => shape.$method($($arg_name)*),
                        Self::Difference(shape) => shape.$method($($arg_name)*),
                        Self::Sketch(shape) => shape.$method($($arg_name)*),
                    }
                }
            )*
        }

        impl Shape for fj::Shape3d {
            $(
                fn $method(&self, $($arg_name: $arg_ty)*) -> $ret {
                    match self {
                        Self::Rotate(shape) => shape.$method($($arg_name)*),
                        Self::Sweep(shape) => shape.$method($($arg_name)*),
                        Self::Union(shape) => shape.$method($($arg_name)*),
                    }
                }
            )*
        }
    };
}

dispatch! {
    bounding_volume() -> AABB;
    faces(tolerance: f64) -> Faces;
    edges() -> Edges;
    vertices() -> Vec<Point>;
}
