use tspf::TspBuilder;
use vehr_alns::ops::{Destroy, GreedyRebuild, RandomDestroy, Rebuild};
use vehr_core::{
    distance::EucDistance,
    reg::NodeRegistry,
    tour::{NodeKind, Tour},
};

fn main() {
    let tsp = TspBuilder::parse_path("./test/data/eilA101.vrp").unwrap();
    let mut reg = NodeRegistry::with_capacity(2, tsp.dim(), 1);

    for (id, pt) in tsp.node_coords() {
        let kind = if tsp.depots().contains(&id) {
            NodeKind::Depot
        } else {
            NodeKind::Request
        };

        let demand = *tsp.demands().get(id).unwrap();
        reg.add(pt.pos().to_owned(), kind, demand);
    }

    reg.compute(&EucDistance::new());

    let mut tour = Tour::new(reg, tsp.capacity());
    tour.init_cw();

    let mut best_dist = tour.total_distance();
    let start_dist = best_dist;

    let start = std::time::Instant::now();
    let mut rd = RandomDestroy::new(0, 0.2);
    let mut gr = GreedyRebuild::new();

    println!("Start: {}", start_dist);
    for _ in 0..100000 {
        let unserved = rd.destroy(&mut tour);
        gr.rebuild(&mut tour, unserved);

        let curr_dist = tour.total_distance();
        if curr_dist < best_dist {
            best_dist = curr_dist;
        }
    }

    let end = std::time::Instant::now() - start;

    println!("Best: {}", best_dist);
    println!("Elapsed: {}", end.as_millis());
    println!("No. routes: {}", tour.n_routes());
}
