use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;

/// A node in a graph with a regular grid.
pub trait GridNode: PartialEq + Eq + Clone + Hash + PartialOrd + Ord {
    /// List the neighbor nodes of this graph node.
    fn neighbors(&self) -> Vec<Self>;
}

/// A pathfinding map structure.
///
/// A Dijkstra map lets you run pathfinding from any graph node it covers
/// towards or away from the target nodes of the map. Currently the structure
/// only supports underlying graphs with a fixed grid graph where the
/// neighbors of each node must be the adjacent grid cells of that node.
pub struct Dijkstra<N> {
    pub weights: HashMap<N, u32>,
}

impl<N: GridNode> Dijkstra<N> {
    /// Create a new Dijkstra map up to limit distance from goals, omitting
    /// nodes for which the is_valid predicate returns false.
    pub fn new<F: Fn(&N) -> bool>(goals: Vec<N>, is_valid: F, limit: u32) -> Dijkstra<N> {
        assert!(!goals.is_empty());

        let mut weights = HashMap::new();
        let mut edge = HashSet::new();

        for n in goals {
            edge.insert(n);
        }

        for dist in 0..(limit) {
            for n in &edge {
                weights.insert(n.clone(), dist);
            }

            let mut new_edge = HashSet::new();
            for n in &edge {
                for m in n.neighbors() {
                    if is_valid(&m) && !weights.contains_key(&m) {
                        new_edge.insert(m);
                    }
                }
            }

            edge = new_edge;

            if edge.is_empty() {
                break;
            }
        }

        Dijkstra { weights }
    }

    /// Return the neighbors of a cell (if any), sorted from downhill to
    /// uphill.
    pub fn sorted_neighbors(&self, node: &N) -> Vec<N> {
        let mut ret = Vec::new();
        for n in &node.neighbors() {
            if let Some(w) = self.weights.get(n) {
                ret.push((w, n.clone()));
            }
        }
        ret.sort_by(|&(w1, _), &(w2, _)| w1.cmp(w2));
        ret.into_iter().map(|(_, n)| n).collect()
    }
}

/// Find A* path in freeform graph.
///
/// The `neighbors` function returns neighboring nodes and their estimated distance from the goal.
/// The search will treat any node whose distance is zero as a goal and return a path leading to
/// it.
pub fn astar_path<N, F>(start: N, end: &N, neighbors: F) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    F: Fn(&N) -> Vec<(N, f32)>,
{
    #[derive(Eq, PartialEq)]
    struct MetricNode<N> {
        value: u32,
        item: N,
        come_from: Option<N>,
    }
    impl<N: Eq> Ord for MetricNode<N> {
        fn cmp(&self, other: &Self) -> Ordering { self.value.cmp(&other.value) }
    }
    impl<N: Eq> PartialOrd for MetricNode<N> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
    }

    fn node<N: Eq>(item: N, dist: f32, come_from: Option<N>) -> MetricNode<N> {
        debug_assert!(dist >= 0.0);
        // Convert dist to integers so we can push MetricNodes into BinaryHeap that expects Ord.
        // The trick here is that non-negative IEEE 754 floats have the same ordering as their
        // binary representations interpreted as integers.
        //
        // Also flip the sign on the value, shorter distance means bigger value, since BinaryHeap
        // returns the largest item first.
        let value = ::std::u32::MAX - dist.to_bits();
        MetricNode {
            item,
            value,
            come_from,
        }
    }

    let mut come_from = HashMap::new();

    let mut open = BinaryHeap::new();
    open.push(node(start.clone(), ::std::f32::MAX, None));

    // Find shortest path.
    let mut goal = loop {
        if let Some(closest) = open.pop() {
            if come_from.contains_key(&closest.item) {
                // Already saw it through a presumably shorter path...
                continue;
            }

            if let Some(from) = closest.come_from {
                come_from.insert(closest.item.clone(), from);
            }

            if &closest.item == end {
                break Some(closest.item);
            }

            for (item, dist) in neighbors(&closest.item) {
                let already_seen = come_from.contains_key(&item) || item == start;
                if already_seen {
                    continue;
                }
                open.push(node(item, dist, Some(closest.item.clone())));
            }
        } else {
            break None;
        }
    };

    // Extract path from the graph structure.
    let mut path = Vec::new();
    while let Some(x) = goal {
        goal = come_from.remove(&x);
        path.push(x);
    }
    path.reverse();

    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_astar() {
        fn neighbors(origin: i32, &x: &i32) -> Vec<(i32, f32)> {
            let mut ret = Vec::with_capacity(2);
            for i in &[-1, 1] {
                let x = x + i;
                ret.push((x, (x - origin).abs() as f32));
            }
            ret
        }

        assert_eq!(Some(vec![8]), astar_path(8, &8, |_| Vec::new()));
        assert_eq!(None, astar_path(8, &12, |_| Vec::new()));

        assert_eq!(Some(vec![8]), astar_path(8, &8, |x| neighbors(8, x)));
        assert_eq!(
            Some(vec![8, 9, 10, 11, 12]),
            astar_path(8, &12, |x| neighbors(8, x))
        );
    }
}
