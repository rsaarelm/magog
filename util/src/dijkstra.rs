use std::hash::Hash;
use std::collections::HashMap;
use std::collections::hash_map::Hasher;
use std::collections::HashSet;

/// A grid node for the Dijkstra map.
pub trait DijkstraNode: Eq+Clone+Hash<Hasher> {
    /// List the neighbor nodes of this graph node.
    fn neighbors(&self) -> Vec<Self>;
}

/// A pathfinding map structure. A Dijkstra map lets you run pathfinding from
/// any graph node it covers towards or away from the target nodes of the map.
/// Currently the structure only supports underlying graphs with a fixed grid graph
/// where the neighbors of each node must be the adjacent grid cells of that
/// node.
pub struct Dijkstra<N> {
    weights: HashMap<N, u32>,
}

impl<N: DijkstraNode> Dijkstra<N> {
    /// Create a new Dijkstra map up to limit distance from goals, omitting
    /// nodes for which the is_valid predicate returns false.
    pub fn new<F: Fn(&N) -> bool>(goals: Vec<N>, is_valid: F, limit: u32) -> Dijkstra<N> {
        assert!(goals.len() > 0);

        let mut weights = HashMap::new();
        let mut edge = HashSet::new();

        for n in goals.into_iter() {
            edge.insert(n);
        }

        for dist in range(0, limit) {
            for n in edge.iter() {
                weights.insert(n.clone(), dist);
            }

            let mut new_edge = HashSet::new();
            for n in edge.iter() {
                for m in n.neighbors().into_iter() {
                    if is_valid(&m) && !weights.contains_key(&m) {
                        new_edge.insert(m);
                    }
                }
            }

            edge = new_edge;
        }

        Dijkstra {
            weights: weights,
        }
    }

    /// Return the neighbors of a cell (if any), sorted from downhill to
    /// uphill.
    pub fn sorted_neighbors(& self, node: &N) -> Vec<N> {
        let mut ret = Vec::new();
        for n in node.neighbors().iter() {
            if let Some(w) = self.weights.get(n) {
                ret.push((w, n.clone()));
            }
        }
        ret.sort_by(|&(w1, _), &(w2, _)| w1.cmp(w2));
        ret.into_iter().map(|(_, n)| n).collect()
    }
}
