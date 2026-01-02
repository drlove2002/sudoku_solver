use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

use super::Graph;

#[derive(Serialize, Deserialize)]
pub struct GraphData {
    nodes: Vec<NodeData>,
    edges: Vec<EdgeData>,
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

    /// Export graph to JSON format for visualization
    pub fn export_to_json(&self, filename: &str) {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Collect all nodes and edges
        for (minigrid_idx, minigrid) in self.minigrids.iter().enumerate() {
            for (perm_idx, node) in minigrid.iter().enumerate() {
                let node_id = format!("mg{}_p{}", minigrid_idx, perm_idx);

                // Add node data
                nodes.push(NodeData {
                    id: node_id.clone(),
                    minigrid: minigrid_idx,
                    perm_id: perm_idx,
                    cells: node.cells().to_vec(),
                    board_position: Self::minigrid_position(minigrid_idx),
                });

                // Add edges (only once, since it's undirected)
                for &(target_mg, target_perm) in &node.compatible {
                    // Only add edge if current minigrid is smaller to avoid duplicates
                    if minigrid_idx < target_mg {
                        edges.push(EdgeData {
                            source: node_id.clone(),
                            target: format!("mg{}_p{}", target_mg, target_perm),
                        });
                    }
                }
            }
        }

        println!("  Nodes: {}", nodes.len());
        println!("  Edges: {}", edges.len());
        let graph_data = GraphData { nodes, edges };
        let json = serde_json::to_string_pretty(&graph_data).expect("Json serialization failed");

        let mut file =
            File::create(filename).unwrap_or_else(|_| panic!("Failed to create {}", filename));
        file.write_all(json.as_bytes())
            .unwrap_or_else(|_| panic!("Failed to write in {}", filename));

        println!("Graph exported to {}", filename);
    }
}
