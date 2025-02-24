use fj_interop::ext::ArrayExt;
use iter_fixed::IntoIteratorFixed;

use crate::{geometry::curve::Curve, objects::Face};

use super::{CurveFaceIntersection, SurfaceSurfaceIntersection};

/// An intersection between two faces
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FaceFaceIntersection {
    /// The intersection curves
    ///
    /// These curves correspond to the input faces, each being the local
    /// representation of the intersection on the respective face's surface.
    ///
    /// They both represent the same global curve.
    pub intersection_curves: [Curve; 2],

    /// The interval of this intersection, in curve coordinates
    ///
    /// These curve coordinates apply to both intersection curves equally.
    pub intersection_intervals: CurveFaceIntersection,
}

impl FaceFaceIntersection {
    /// Compute the intersections between two faces
    pub fn compute(faces: [&Face; 2]) -> Option<Self> {
        let surfaces = faces.map(|face| face.surface().clone());

        let intersection_curves =
            match SurfaceSurfaceIntersection::compute(surfaces) {
                Some(intersection) => intersection.intersection_curves,
                None => return None,
            };

        let curve_face_intersections = intersection_curves
            .each_ref_ext()
            .into_iter_fixed()
            .zip(faces)
            .map(|(curve, face)| CurveFaceIntersection::compute(curve, face))
            .collect::<[_; 2]>();

        let intersection_intervals = {
            let [a, b] = curve_face_intersections;
            a.merge(&b)
        };

        if intersection_intervals.is_empty() {
            return None;
        }

        Some(Self {
            intersection_curves,
            intersection_intervals,
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        algorithms::intersect::CurveFaceIntersection,
        builder::{CycleBuilder, FaceBuilder},
        geometry::curve::Curve,
        services::Services,
    };

    use super::FaceFaceIntersection;

    #[test]
    fn compute_no_intersection() {
        let mut services = Services::new();

        #[rustfmt::skip]
        let points = [
            [1., 1.],
            [2., 1.],
            [2., 2.],
            [1., 2.],
        ];
        let [a, b] = [
            services.objects.surfaces.xy_plane(),
            services.objects.surfaces.xz_plane(),
        ]
        .map(|surface| {
            FaceBuilder::new(surface)
                .with_exterior(CycleBuilder::polygon(
                    points,
                    &mut services.objects,
                ))
                .build(&mut services.objects)
        });

        let intersection = FaceFaceIntersection::compute([&a, &b]);

        assert!(intersection.is_none());
    }

    #[test]
    fn compute_one_intersection() {
        let mut services = Services::new();

        #[rustfmt::skip]
        let points = [
            [-1., -1.],
            [ 1., -1.],
            [ 1.,  1.],
            [-1.,  1.],
        ];
        let surfaces = [
            services.objects.surfaces.xy_plane(),
            services.objects.surfaces.xz_plane(),
        ];
        let [a, b] = surfaces.clone().map(|surface| {
            FaceBuilder::new(surface)
                .with_exterior(CycleBuilder::polygon(
                    points,
                    &mut services.objects,
                ))
                .build(&mut services.objects)
        });

        let intersection = FaceFaceIntersection::compute([&a, &b]);

        let expected_curves = surfaces.map(|_| {
            let (path, _) = Curve::line_from_points([[0., 0.], [1., 0.]]);
            path
        });
        let expected_intervals =
            CurveFaceIntersection::from_intervals([[[-1.], [1.]]]);
        assert_eq!(
            intersection,
            Some(FaceFaceIntersection {
                intersection_curves: expected_curves,
                intersection_intervals: expected_intervals
            })
        );
    }
}
