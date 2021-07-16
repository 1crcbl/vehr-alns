use std::collections::HashSet;

use oorandom::Rand64;
use vehr_core::tour::{Node, NodeKind, Route, Tour};

pub trait Destroy {
    fn destroy(&mut self, tour: &mut Tour) -> Vec<Node>;
}

#[derive(Clone, Debug)]
pub struct RandomDestroy {
    pop: f64,
    rng: Rand64,
}

impl RandomDestroy {
    pub fn new(seed: u128, pop: f64) -> Self {
        Self {
            pop,
            rng: Rand64::new(seed),
        }
    }
}

impl Destroy for RandomDestroy {
    fn destroy(&mut self, tour: &mut Tour) -> Vec<Node> {
        let n_nodes = tour.n_nodes() as u64;
        let mut rem_cnt = (self.pop * n_nodes as f64).ceil() as usize;

        let mut set = HashSet::new();
        let mut result = Vec::with_capacity(rem_cnt);

        while rem_cnt > 0 {
            let rng_idx = self.rng.rand_range(0..n_nodes) as usize;

            if !set.contains(&rng_idx) {
                set.insert(rng_idx);
                if let Some(node) = tour.node_mut(rng_idx) {
                    if node.kind() != NodeKind::Depot {
                        Route::eject(node);
                        rem_cnt -= 1;
                        result.push(node.clone());
                    }
                }
            }
        }

        tour.drop_empty();

        result
    }
}

pub trait Rebuild {
    fn rebuild(&mut self, tour: &mut Tour, unserved: Vec<Node>);
}

pub struct GreedyRebuild {}

impl GreedyRebuild {
    pub fn new() -> Self {
        Self {}
    }
}

impl Rebuild for GreedyRebuild {
    fn rebuild(&mut self, tour: &mut Tour, mut unserved: Vec<Node>) {
        for mut node in unserved.drain(..) {
            let mut min_dist = std::f64::MAX;
            let mut pivot = None;

            for route in tour.route_iter() {
                if route.check_capacity(node.demand()) {
                    for arc in route.arc_iter() {
                        let arc_dist = tour.distance(arc.tail(), arc.head());
                        let new_arc_dist =
                            tour.distance(arc.tail(), &node) + tour.distance(&node, arc.head());
                        let delta = new_arc_dist - arc_dist;

                        if delta < min_dist {
                            min_dist = delta;
                            pivot = Some(arc.tail().clone());
                        }
                    }
                }
            }

            match pivot {
                Some(mut pivot) => {
                    Route::insert_back(&mut pivot, &mut node);
                }
                None => {
                    let new_route = tour.new_route();
                    new_route.push_back(&node);
                }
            }
        }
    }
}

impl Default for GreedyRebuild {
    fn default() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use vehr_core::{
        distance::LowerColDist,
        reg::NodeRegistry,
        tour::{NodeKind, Tour},
    };

    use crate::ops::{Destroy, GreedyRebuild, RandomDestroy, Rebuild};

    #[test]
    fn test_random_greedy() {
        let mut reg = NodeRegistry::with_capacity(0, 7, 1);
        reg.add(vec![0.; 0], NodeKind::Depot, 0.);
        reg.add(vec![0.; 0], NodeKind::Request, 1.);
        reg.add(vec![0.; 0], NodeKind::Request, 1.);
        reg.add(vec![0.; 0], NodeKind::Request, 1.);
        reg.add(vec![0.; 0], NodeKind::Request, 1.);
        reg.add(vec![0.; 0], NodeKind::Request, 1.);
        reg.add(vec![0.; 0], NodeKind::Request, 1.);

        let lcd = LowerColDist::new(
            7,
            vec![
                10., 20., 25., 25., 20., 10., 12., 20., 25., 30., 20., 10., 11., 22., 30., 2., 11.,
                25., 10., 20., 12.,
            ],
        );

        reg.compute(&lcd);

        let mut tour = Tour::new(reg, 3.);
        tour.init_cw();

        let mut rd = RandomDestroy::new(0, 0.2);
        let unserved = rd.destroy(&mut tour);
        assert!(unserved.len() > 0);

        let mut gr = GreedyRebuild::new();
        gr.rebuild(&mut tour, unserved);
    }
}
