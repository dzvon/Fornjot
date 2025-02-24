use std::{collections::HashMap, iter::repeat};

use fj_math::{Point, Scalar};

use crate::{
    geometry::surface::SurfaceGeometry,
    objects::{HalfEdge, Shell, Surface},
    storage::{Handle, ObjectId},
};

use super::{Validate, ValidationConfig, ValidationError};

impl Validate for Shell {
    fn validate_with_config(
        &self,
        config: &ValidationConfig,
        errors: &mut Vec<ValidationError>,
    ) {
        ShellValidationError::validate_edges_coincident(self, config, errors);
        ShellValidationError::validate_watertight(self, config, errors);
    }
}

/// [`Shell`] validation failed
#[derive(Clone, Debug, thiserror::Error)]
pub enum ShellValidationError {
    /// [`Shell`] contains global_edges not referred to by two half_edges
    #[error("Shell is not watertight")]
    NotWatertight,

    /// [`Shell`] contains half_edges that are coincident, but refer to different global_edges
    #[error(
        "`Shell` contains `HalfEdge`s that are coincident but refer to \
        different `GlobalEdge`s\n\
        Edge 1: {0:#?}\n\
        Edge 2: {1:#?}"
    )]
    CoincidentEdgesNotIdentical(Handle<HalfEdge>, Handle<HalfEdge>),

    /// [`Shell`] contains half_edges that are identical, but do not coincide
    #[error(
        "Shell contains HalfEdges that are identical but do not coincide\n\
        Edge 1: {edge_1:#?}\n\
        Surface for edge 1: {surface_1:#?}\n\
        Edge 2: {edge_2:#?}\n\
        Surface for edge 2: {surface_2:#?}"
    )]
    IdenticalEdgesNotCoincident {
        /// The first edge
        edge_1: Handle<HalfEdge>,

        /// The surface that the first edge is on
        surface_1: Handle<Surface>,

        /// The second edge
        edge_2: Handle<HalfEdge>,

        /// The surface that the second edge is on
        surface_2: Handle<Surface>,
    },
}

/// Sample two edges at various (currently 3) points in 3D along them.
///
/// Returns an [`Iterator`] of the distance at each sample.
fn distances(
    config: &ValidationConfig,
    (edge1, surface1): (Handle<HalfEdge>, Handle<Surface>),
    (edge2, surface2): (Handle<HalfEdge>, Handle<Surface>),
) -> impl Iterator<Item = Scalar> {
    fn sample(
        percent: f64,
        (edge, surface): (&Handle<HalfEdge>, SurfaceGeometry),
    ) -> Point<3> {
        let boundary = edge.boundary();
        let path_coords = boundary[0] + (boundary[1] - boundary[0]) * percent;
        let surface_coords = edge.curve().point_from_path_coords(path_coords);
        surface.point_from_surface_coords(surface_coords)
    }

    // Check whether start positions do not match. If they don't treat second edge as flipped
    let flip = sample(0.0, (&edge1, surface1.geometry()))
        .distance_to(&sample(0.0, (&edge2, surface2.geometry())))
        > config.identical_max_distance;

    // Three samples (start, middle, end), are enough to detect weather lines
    // and circles match. If we were to add more complicated curves, this might
    // need to change.
    let sample_count = 3;
    let step = 1.0 / (sample_count as f64 - 1.0);

    let mut distances = Vec::new();
    for i in 0..sample_count {
        let percent = i as f64 * step;
        let sample1 = sample(percent, (&edge1, surface1.geometry()));
        let sample2 = sample(
            if flip { 1.0 - percent } else { percent },
            (&edge2, surface2.geometry()),
        );
        distances.push(sample1.distance_to(&sample2))
    }
    distances.into_iter()
}

impl ShellValidationError {
    fn validate_edges_coincident(
        shell: &Shell,
        config: &ValidationConfig,
        errors: &mut Vec<ValidationError>,
    ) {
        let edges_and_surfaces: Vec<_> = shell
            .faces()
            .into_iter()
            .flat_map(|face| {
                face.all_cycles()
                    .flat_map(|cycle| cycle.half_edges().cloned())
                    .zip(repeat(face.surface().clone()))
            })
            .collect();

        // This is O(N^2) which isn't great, but we can't use a HashMap since we
        // need to deal with float inaccuracies. Maybe we could use some smarter
        // data-structure like an octree.
        for edge in &edges_and_surfaces {
            for other_edge in &edges_and_surfaces {
                let id = edge.0.global_form().id();
                let other_id = other_edge.0.global_form().id();
                let identical = id == other_id;
                match identical {
                    true => {
                        // All points on identical curves should be within
                        // identical_max_distance, so we shouldn't have any
                        // greater than the max
                        if distances(config, edge.clone(), other_edge.clone())
                            .any(|d| d > config.identical_max_distance)
                        {
                            errors.push(
                                Self::IdenticalEdgesNotCoincident {
                                    edge_1: edge.0.clone(),
                                    surface_1: edge.1.clone(),
                                    edge_2: other_edge.0.clone(),
                                    surface_2: other_edge.1.clone(),
                                }
                                .into(),
                            )
                        }
                    }
                    false => {
                        // If all points on distinct curves are within
                        // distinct_min_distance, that's a problem.
                        if distances(config, edge.clone(), other_edge.clone())
                            .all(|d| d < config.distinct_min_distance)
                        {
                            errors.push(
                                Self::CoincidentEdgesNotIdentical(
                                    edge.0.clone(),
                                    other_edge.0.clone(),
                                )
                                .into(),
                            )
                        }
                    }
                }
            }
        }
    }

    fn validate_watertight(
        shell: &Shell,
        _: &ValidationConfig,
        errors: &mut Vec<ValidationError>,
    ) {
        let faces = shell.faces();
        let mut half_edge_to_faces: HashMap<ObjectId, usize> = HashMap::new();
        for face in faces {
            for cycle in face.all_cycles() {
                for half_edge in cycle.half_edges() {
                    let id = half_edge.global_form().id();
                    let entry = half_edge_to_faces.entry(id);
                    *entry.or_insert(0) += 1;
                }
            }
        }

        // Each global edge should have exactly two half edges that are part of the shell
        if half_edge_to_faces.iter().any(|(_, c)| *c != 2) {
            errors.push(Self::NotWatertight.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_contains_err,
        objects::{GlobalEdge, Shell},
        operations::{
            BuildShell, Insert, UpdateCycle, UpdateFace, UpdateHalfEdge,
            UpdateShell,
        },
        services::Services,
        validate::{shell::ShellValidationError, Validate, ValidationError},
    };

    #[test]
    fn coincident_not_identical() -> anyhow::Result<()> {
        let mut services = Services::new();

        let valid = Shell::tetrahedron(
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            &mut services.objects,
        );
        let invalid = valid.shell.update_face(&valid.face_abc, |face| {
            face.update_exterior(|cycle| {
                cycle
                    .update_half_edge(0, |half_edge| {
                        let global_form =
                            GlobalEdge::new().insert(&mut services.objects);
                        half_edge
                            .update_global_form(global_form)
                            .insert(&mut services.objects)
                    })
                    .insert(&mut services.objects)
            })
            .insert(&mut services.objects)
        });

        valid.shell.validate_and_return_first_error()?;
        assert_contains_err!(
            invalid,
            ValidationError::Shell(
                ShellValidationError::CoincidentEdgesNotIdentical(..)
            )
        );

        Ok(())
    }
    #[test]
    fn shell_not_watertight() -> anyhow::Result<()> {
        let mut services = Services::new();

        let valid = Shell::tetrahedron(
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            &mut services.objects,
        );
        let invalid = valid.shell.remove_face(&valid.face_abc);

        valid.shell.validate_and_return_first_error()?;
        assert_contains_err!(
            invalid,
            ValidationError::Shell(ShellValidationError::NotWatertight)
        );

        Ok(())
    }
}
