use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

use super::Graph;
use crate::solver::report::Solution;

#[derive(Serialize, Deserialize)]
pub struct GraphData {
    nodes: Vec<NodeData>,
    edges: Vec<EdgeData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    solutions: Option<Vec<SolutionData>>,
}

#[derive(Serialize, Deserialize)]
struct NodeData {
    id: String,
    minigrid: usize,
    perm_id: usize,
    cells: Vec<u8>,
    board_position: String,
}

#[derive(Serialize, Deserialize)]
struct EdgeData {
    source: String,
    target: String,
}

#[derive(Serialize, Deserialize)]
struct SolutionData {
    board: Vec<Vec<u8>>,
}

impl<const K: usize, const N: usize> Graph<K, N> {
    /// Get position label for a minigrid (e.g., "top-left", "middle-center")
    fn minigrid_position(minigrid_idx: usize) -> String {
        let row = minigrid_idx / K;
        let col = minigrid_idx % K;

        let row_name = match row {
            0 => "top",
            r if r == K - 1 => "bottom",
            _ => "middle",
        };

        let col_name = match col {
            0 => "left",
            c if c == K - 1 => "right",
            _ => "center",
        };

        format!("{}-{}", row_name, col_name)
    }

    /// Patch an existing graph JSON with solved board data
    pub fn patch_json_with_solutions(&self, path: &str, solutions: &[Solution<N>]) {
        let raw = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read {}", path));
        let mut graph_data: GraphData =
            serde_json::from_str(&raw).expect("Failed to parse graph JSON");

        graph_data.solutions = Some(
            solutions
                .iter()
                .map(|s| SolutionData {
                    board: s.board.cells.iter().map(|row| row.to_vec()).collect(),
                })
                .collect(),
        );

        let json = serde_json::to_string_pretty(&graph_data).expect("Json serialization failed");
        let mut file =
            std::fs::File::create(path).unwrap_or_else(|_| panic!("Failed to create {}", path));
        file.write_all(json.as_bytes())
            .unwrap_or_else(|_| panic!("Failed to write in {}", path));

        println!("Patched graph JSON with {} solution(s)", solutions.len());
    }

    /// Export graph to JSON format for visualization
    pub fn export_to_json(&self, filename: &str) {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Collect all nodes and edges
        for (minigrid_idx, minigrid) in self.minigrids.iter().enumerate() {
            for (perm_idx, _) in minigrid.iter().enumerate() {
                let node_id = format!("mg{}_p{}", minigrid_idx, perm_idx);

                // Add node data
                nodes.push(NodeData {
                    id: node_id.clone(),
                    minigrid: minigrid_idx,
                    perm_id: perm_idx,
                    cells: self.permutation_cells(minigrid_idx, perm_idx).to_vec(),
                    board_position: Self::minigrid_position(minigrid_idx),
                });

                // Add edges (only once, since it's undirected)
                for target_mg in (minigrid_idx + 1)..N {
                    if let Some(target_set) = self.compatible_set(minigrid_idx, perm_idx, target_mg)
                    {
                        for target_perm in target_set.iter_ones() {
                            edges.push(EdgeData {
                                source: node_id.clone(),
                                target: format!("mg{}_p{}", target_mg, target_perm),
                            });
                        }
                    }
                }
            }
        }

        println!("  Nodes: {}", nodes.len());
        println!("  Edges: {}", edges.len());
        let graph_data = GraphData { nodes, edges, solutions: None };
        let json = serde_json::to_string_pretty(&graph_data).expect("Json serialization failed");

        let mut file =
            std::fs::File::create(filename).unwrap_or_else(|_| panic!("Failed to create {}", filename));
        file.write_all(json.as_bytes())
            .unwrap_or_else(|_| panic!("Failed to write in {}", filename));

        println!("Graph exported to {}", filename);
    }
}
