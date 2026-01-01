use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
// use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum GraphError {
    #[error("Circular dependency detected: {0}")]
    Cycle(String),
}

/// A Dependency Graph that supports batched parallel execution.
pub struct ExecutionDag {
    graph: DiGraph<String, ()>,
    // Map package name to NodeIndex for quick lookups
    node_map: HashMap<String, NodeIndex>,
}

impl ExecutionDag {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, name: impl Into<String>) -> NodeIndex {
        let name = name.into();
        if let Some(&idx) = self.node_map.get(&name) {
            idx
        } else {
            let idx = self.graph.add_node(name.clone());
            self.node_map.insert(name, idx);
            idx
        }
    }

    pub fn add_dependency(&mut self, from: &str, to: &str) {
        // A depends on B -> B must be installed before A.
        // Edge: B -> A
        let from_idx = self.add_node(from);
        let to_idx = self.add_node(to);

        // Avoid duplicate edges
        if self.graph.find_edge(to_idx, from_idx).is_none() {
            self.graph.add_edge(to_idx, from_idx, ());
        }
    }

    /// Returns a simple linear installation order.
    pub fn resolve(&self) -> Result<Vec<String>, GraphError> {
        match toposort(&self.graph, None) {
            Ok(nodes) => {
                let sorted_names: Vec<String> =
                    nodes.iter().map(|&idx| self.graph[idx].clone()).collect();
                Ok(sorted_names)
            }
            Err(cycle) => {
                let node_weight = &self.graph[cycle.node_id()];
                Err(GraphError::Cycle(node_weight.clone()))
            }
        }
    }

    /// Returns the installation order in batches (layers).
    /// Each batch contains nodes that can be processed in parallel.
    pub fn resolve_batched(&self) -> Result<Vec<Vec<String>>, GraphError> {
        let mut batches = Vec::new();
        let mut in_degrees: HashMap<NodeIndex, usize> = self
            .graph
            .node_indices()
            .map(|idx| {
                (
                    idx,
                    self.graph.edges_directed(idx, Direction::Incoming).count(),
                )
            })
            .collect();

        let mut processed_count = 0;
        let total_nodes = self.graph.node_count();

        while processed_count < total_nodes {
            // Find all nodes with in-degree 0 that haven't been processed
            let current_batch: Vec<NodeIndex> = in_degrees
                .iter()
                .filter(|(_, &deg)| deg == 0)
                .map(|(&idx, _)| idx)
                .collect();

            if current_batch.is_empty() {
                // If we still have nodes but none have in-degree 0, there's a cycle.
                // We pick the first unprocessed node to report in the error.
                let cycle_node_idx = in_degrees.keys().next().unwrap();
                return Err(GraphError::Cycle(self.graph[*cycle_node_idx].clone()));
            }

            let mut batch_names = Vec::new();
            for &node_idx in &current_batch {
                batch_names.push(self.graph[node_idx].clone());

                // Remove outgoing edges conceptually: decrement in-degree of neighbors
                for neighbor in self.graph.neighbors_directed(node_idx, Direction::Outgoing) {
                    if let Some(deg) = in_degrees.get_mut(&neighbor) {
                        *deg -= 1;
                    }
                }

                // Remove processed node from tracking
                in_degrees.remove(&node_idx);
                processed_count += 1;
            }

            batches.push(batch_names);
        }

        Ok(batches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_dag() {
        let mut dag = ExecutionDag::new();
        dag.add_dependency("A", "B"); // A depends on B
        dag.add_dependency("B", "C"); // B depends on C

        let res = dag.resolve().unwrap();
        assert_eq!(res, vec!["C", "B", "A"]);

        let batched = dag.resolve_batched().unwrap();
        assert_eq!(batched, vec![vec!["C"], vec!["B"], vec!["A"]]);
    }

    #[test]
    fn test_parallel_dag() {
        let mut dag = ExecutionDag::new();
        // A -> B, C
        // B -> D
        // C -> D
        dag.add_dependency("A", "B");
        dag.add_dependency("A", "C");
        dag.add_dependency("B", "D");
        dag.add_dependency("C", "D");

        let batched = dag.resolve_batched().unwrap();
        // Layer 0: D
        // Layer 1: B, C (order doesn't matter much in Vec)
        // Layer 2: A
        assert_eq!(batched.len(), 3);
        assert_eq!(batched[0], vec!["D"]);
        assert!(batched[1].contains(&"B".to_string()));
        assert!(batched[1].contains(&"C".to_string()));
        assert_eq!(batched[2], vec!["A"]);
    }

    #[test]
    fn test_cycle() {
        let mut dag = ExecutionDag::new();
        dag.add_dependency("A", "B");
        dag.add_dependency("B", "A");

        let res = dag.resolve();
        assert!(res.is_err());

        let batched = dag.resolve_batched();
        assert!(batched.is_err());
    }
}
