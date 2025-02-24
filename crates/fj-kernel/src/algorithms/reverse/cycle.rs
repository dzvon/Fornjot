use itertools::Itertools;

use crate::{
    objects::{Cycle, HalfEdge, Objects},
    operations::Insert,
    services::Service,
    storage::Handle,
};

use super::Reverse;

impl Reverse for Handle<Cycle> {
    fn reverse(self, objects: &mut Service<Objects>) -> Self {
        let mut edges = self
            .half_edges()
            .cloned()
            .circular_tuple_windows()
            .map(|(current, next)| {
                let boundary = {
                    let [a, b] = current.boundary();
                    [b, a]
                };

                HalfEdge::new(
                    current.curve(),
                    boundary,
                    next.start_vertex().clone(),
                    current.global_form().clone(),
                )
                .insert(objects)
            })
            .collect::<Vec<_>>();

        edges.reverse();

        Cycle::new(edges).insert(objects)
    }
}
