mod node;
mod visualize;

use crate::types::bitstring::FixedBitSet;
use log::info;
pub use node::{PermId, PermutationNode};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GeneratedMinigrid<const N: usize, const K: usize> {
    pub nodes: Vec<PermutationNode<N, K>>,
    pub payloads: Vec<[u8; N]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Row,
    Col,
    Not,
}

impl Relation {
    #[inline]
    pub fn from_mask(mask: usize) -> Self {
        const LUT: [Relation; 4] = [Relation::Not, Relation::Row, Relation::Col, Relation::Not];
        LUT[mask & 3]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairLookup {
    pair_idx: usize,
    reversed: bool,
}

pub type SigId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaskSignature<const K: usize> {
    pub masks: [u32; K],
}

#[derive(Debug, Clone)]
pub struct AxisIndex<const K: usize> {
    pub signatures: Vec<MaskSignature<K>>,
    pub perm_to_sig: Vec<SigId>,
    pub sig_to_perms: Vec<FixedBitSet>,
}

#[derive(Debug, Clone)]
pub struct PairAdj {
    left_mg: usize,
    right_mg: usize,
    relation: Relation,
    left_sig_to_right: Vec<FixedBitSet>,
    right_sig_to_left: Vec<FixedBitSet>,
}

pub struct Graph<const K: usize, const N: usize> {
    minigrids: [Vec<PermutationNode<N, K>>; N],
    payloads: [Vec<[u8; N]>; N],
    row_indexes: [AxisIndex<K>; N],
    col_indexes: [AxisIndex<K>; N],
    pair_tables: Vec<PairAdj>,
    pair_lookup: [[Option<PairLookup>; N]; N],
}

impl<const K: usize, const N: usize> Graph<K, N> {
    pub fn new(generated: [GeneratedMinigrid<N, K>; N]) -> Self {
        let mut minigrid_vecs = Vec::with_capacity(N);
        let mut payload_vecs = Vec::with_capacity(N);
        for generated_mg in generated {
            minigrid_vecs.push(generated_mg.nodes);
            payload_vecs.push(generated_mg.payloads);
        }

        let minigrids: [Vec<PermutationNode<N, K>>; N] = minigrid_vecs
            .try_into()
            .expect("generated minigrid count must match board size");
        let payloads: [Vec<[u8; N]>; N] = payload_vecs
            .try_into()
            .expect("generated payload count must match board size");
        let row_indexes =
            std::array::from_fn(|mg_id| build_axis_index::<N, K>(&minigrids[mg_id], true));
        let col_indexes =
            std::array::from_fn(|mg_id| build_axis_index::<N, K>(&minigrids[mg_id], false));

        Self {
            minigrids,
            payloads,
            row_indexes,
            col_indexes,
            pair_tables: Vec::new(),
            pair_lookup: [[None; N]; N],
        }
    }

    #[inline]
    pub fn relationship_between(a: usize, b: usize) -> Relation {
        let row_eq = (((a / K) ^ (b / K)) == 0) as usize;
        let col_eq = (((a % K) ^ (b % K)) == 0) as usize;
        Relation::from_mask(row_eq | (col_eq << 1))
    }

    #[inline]
    pub fn relationship(&self, a: usize, b: usize) -> Relation {
        Self::relationship_between(a, b)
    }

    pub fn create_edges(&mut self) {
        self.pair_tables.clear();

        let mut total_edges = 0usize;
        for left_mg in 0..N {
            for right_mg in (left_mg + 1)..N {
                let relation = Self::relationship_between(left_mg, right_mg);
                if relation == Relation::Not {
                    continue;
                }

                let left_index = match relation {
                    Relation::Row => &self.row_indexes[left_mg],
                    Relation::Col => &self.col_indexes[left_mg],
                    Relation::Not => unreachable!("unrelated minigrid pairs are skipped"),
                };
                let right_index = match relation {
                    Relation::Row => &self.row_indexes[right_mg],
                    Relation::Col => &self.col_indexes[right_mg],
                    Relation::Not => unreachable!("unrelated minigrid pairs are skipped"),
                };

                let left_len = self.minigrids[left_mg].len();
                let right_len = self.minigrids[right_mg].len();
                let right_word_count = right_len.div_ceil(64);
                let left_word_count = left_len.div_ceil(64);

                let mut left_sig_to_right_words =
                    vec![vec![0u64; right_word_count]; left_index.signatures.len()];
                let mut right_sig_to_left_words =
                    vec![vec![0u64; left_word_count]; right_index.signatures.len()];
                let mut edges_added = 0usize;

                for (left_sig_id, left_sig) in left_index.signatures.iter().enumerate() {
                    for (right_sig_id, right_sig) in right_index.signatures.iter().enumerate() {
                        if signatures_compatible(left_sig, right_sig) {
                            or_words(
                                &mut left_sig_to_right_words[left_sig_id],
                                right_index.sig_to_perms[right_sig_id].iter(),
                            );
                            or_words(
                                &mut right_sig_to_left_words[right_sig_id],
                                left_index.sig_to_perms[left_sig_id].iter(),
                            );
                            edges_added += left_index.sig_to_perms[left_sig_id].count_ones()
                                * right_index.sig_to_perms[right_sig_id].count_ones();
                        }
                    }
                }

                info!(
                    "MG{}-MG{} ({:?}): {} edge(s)",
                    left_mg, right_mg, relation, edges_added
                );
                total_edges += edges_added;

                self.pair_tables.push(PairAdj {
                    left_mg,
                    right_mg,
                    relation,
                    left_sig_to_right: left_sig_to_right_words
                        .into_iter()
                        .map(|words| FixedBitSet::from_words(words, right_len))
                        .collect(),
                    right_sig_to_left: right_sig_to_left_words
                        .into_iter()
                        .map(|words| FixedBitSet::from_words(words, left_len))
                        .collect(),
                });
            }
        }

        self.pair_lookup = Self::build_pair_lookup(&self.pair_tables);
        info!("Total edges created: {}", total_edges);
    }

    pub fn retain_permutations(&mut self, keep: &[Vec<usize>; N]) {
        let mut old_minigrids =
            std::mem::replace(&mut self.minigrids, std::array::from_fn(|_| Vec::new()));
        let mut old_payloads =
            std::mem::replace(&mut self.payloads, std::array::from_fn(|_| Vec::new()));

        let keep_mask: [Vec<bool>; N] = std::array::from_fn(|mg_id| {
            let mut mask = vec![false; old_minigrids[mg_id].len()];
            for &old_idx in &keep[mg_id] {
                mask[old_idx] = true;
            }
            mask
        });

        let mut new_minigrids = std::array::from_fn(|_| Vec::new());
        let mut new_payloads = std::array::from_fn(|_| Vec::new());
        for mg_id in 0..N {
            new_minigrids[mg_id] = Vec::with_capacity(keep[mg_id].len());
            new_payloads[mg_id] = Vec::with_capacity(keep[mg_id].len());

            for (old_idx, (mut node, payload)) in std::mem::take(&mut old_minigrids[mg_id])
                .into_iter()
                .zip(std::mem::take(&mut old_payloads[mg_id]))
                .enumerate()
            {
                if keep_mask[mg_id][old_idx] {
                    let payload_idx = PermId::try_from(new_payloads[mg_id].len())
                        .expect("payload count must fit into u32");
                    node.set_payload_idx(payload_idx);
                    new_payloads[mg_id].push(payload);
                    new_minigrids[mg_id].push(node);
                }
            }
        }

        self.minigrids = new_minigrids;
        self.payloads = new_payloads;
        self.row_indexes =
            std::array::from_fn(|mg_id| build_axis_index::<N, K>(&self.minigrids[mg_id], true));
        self.col_indexes =
            std::array::from_fn(|mg_id| build_axis_index::<N, K>(&self.minigrids[mg_id], false));
        self.create_edges();
    }

    pub fn permutation_degrees(&self, mg_id: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.minigrids[mg_id].len())
            .map(move |perm_id| (perm_id, self.permutation_degree(mg_id, perm_id)))
    }

    pub fn total_permutations(&self) -> usize {
        self.minigrids.iter().map(Vec::len).sum()
    }

    pub fn total_edges(&self) -> usize {
        self.pair_tables
            .iter()
            .map(|pair| {
                let left_index = match pair.relation {
                    Relation::Row => &self.row_indexes[pair.left_mg],
                    Relation::Col => &self.col_indexes[pair.left_mg],
                    Relation::Not => unreachable!("unrelated minigrid pairs are skipped"),
                };

                pair.left_sig_to_right
                    .iter()
                    .enumerate()
                    .map(|(sig_id, edges)| {
                        edges.count_ones() * left_index.sig_to_perms[sig_id].count_ones()
                    })
                    .sum::<usize>()
            })
            .sum()
    }

    pub fn permutation_count(&self, mg_id: usize) -> usize {
        self.minigrids[mg_id].len()
    }

    pub fn permutation_degree(&self, mg_id: usize, perm_id: usize) -> usize {
        let mut degree = 0usize;
        for target_mg in 0..N {
            if let Some(edges) = self.compatible_set(mg_id, perm_id, target_mg) {
                degree += edges.count_ones();
            }
        }
        degree
    }

    pub fn compatible_set(
        &self,
        mg_id: usize,
        perm_id: usize,
        target_mg: usize,
    ) -> Option<&FixedBitSet> {
        let lookup = self.pair_lookup[mg_id][target_mg]?;
        let pair = &self.pair_tables[lookup.pair_idx];
        let axis_index = match pair.relation {
            Relation::Row => &self.row_indexes[mg_id],
            Relation::Col => &self.col_indexes[mg_id],
            Relation::Not => unreachable!("unrelated minigrid pairs are skipped"),
        };
        let sig_id = usize::try_from(axis_index.perm_to_sig[perm_id])
            .expect("signature id must fit into usize");

        if lookup.reversed {
            pair.right_sig_to_left.get(sig_id)
        } else {
            pair.left_sig_to_right.get(sig_id)
        }
    }

    pub fn permutation_cells(&self, mg_id: usize, perm_id: usize) -> &[u8; N] {
        let payload_idx = usize::try_from(self.minigrids[mg_id][perm_id].payload_idx())
            .expect("payload index must fit into usize");
        &self.payloads[mg_id][payload_idx]
    }

    pub fn pair_count(&self) -> usize {
        self.pair_tables.len()
    }

    pub fn memory_usage(&self) -> u64 {
        let mut bytes = 0u64;

        // PermutationNodes
        let node_size =
            std::mem::size_of::<PermutationNode<N, K>>();
        for mg in &self.minigrids {
            bytes += (mg.capacity() * node_size) as u64;
        }

        // Payloads: [u8; N] each
        for p in &self.payloads {
            bytes += (p.capacity() * N) as u64;
        }

        // Pair adjacency tables (FixedBitSet heap)
        for pair in &self.pair_tables {
            for bs in &pair.left_sig_to_right {
                bytes += bs.memory_bytes() as u64;
            }
            for bs in &pair.right_sig_to_left {
                bytes += bs.memory_bytes() as u64;
            }
        }

        // Axis indexes (all the FixedBitSets)
        for idx in &self.row_indexes {
            bytes += (idx.perm_to_sig.capacity() * std::mem::size_of::<SigId>()) as u64;
            for bs in &idx.sig_to_perms {
                bytes += bs.memory_bytes() as u64;
            }
        }
        for idx in &self.col_indexes {
            bytes += (idx.perm_to_sig.capacity() * std::mem::size_of::<SigId>()) as u64;
            for bs in &idx.sig_to_perms {
                bytes += bs.memory_bytes() as u64;
            }
        }

        bytes
    }

    fn build_pair_lookup(pair_tables: &[PairAdj]) -> [[Option<PairLookup>; N]; N] {
        let mut lookup = [[None; N]; N];
        for (pair_idx, pair) in pair_tables.iter().enumerate() {
            lookup[pair.left_mg][pair.right_mg] = Some(PairLookup {
                pair_idx,
                reversed: false,
            });
            lookup[pair.right_mg][pair.left_mg] = Some(PairLookup {
                pair_idx,
                reversed: true,
            });
        }
        lookup
    }
}

#[inline(always)]
fn row_signature<const N: usize, const K: usize>(node: &PermutationNode<N, K>) -> MaskSignature<K> {
    MaskSignature {
        masks: std::array::from_fn(|i| node.row_masks[i].raw()),
    }
}

#[inline(always)]
fn col_signature<const N: usize, const K: usize>(node: &PermutationNode<N, K>) -> MaskSignature<K> {
    MaskSignature {
        masks: std::array::from_fn(|i| node.col_masks[i].raw()),
    }
}

#[inline(always)]
fn signatures_compatible<const K: usize>(a: &MaskSignature<K>, b: &MaskSignature<K>) -> bool {
    for i in 0..K {
        if (a.masks[i] & b.masks[i]) != 0 {
            return false;
        }
    }
    true
}

fn build_axis_index<const N: usize, const K: usize>(
    nodes: &[PermutationNode<N, K>],
    use_rows: bool,
) -> AxisIndex<K> {
    let mut signatures = Vec::new();
    let mut perm_to_sig = Vec::with_capacity(nodes.len());
    let mut signature_map = HashMap::<MaskSignature<K>, usize>::new();
    let word_count = nodes.len().div_ceil(64);
    let mut sig_to_perm_words: Vec<Vec<u64>> = Vec::new();

    for (perm_id, node) in nodes.iter().enumerate() {
        let signature = if use_rows {
            row_signature(node)
        } else {
            col_signature(node)
        };

        let sig_id = *signature_map.entry(signature).or_insert_with(|| {
            let next_sig_id = signatures.len();
            signatures.push(signature);
            sig_to_perm_words.push(vec![0u64; word_count]);
            next_sig_id
        });

        perm_to_sig.push(SigId::try_from(sig_id).expect("signature count must fit into u32"));
        set_bit(&mut sig_to_perm_words[sig_id], perm_id);
    }

    AxisIndex {
        signatures,
        perm_to_sig,
        sig_to_perms: sig_to_perm_words
            .into_iter()
            .map(|words| FixedBitSet::from_words(words, nodes.len()))
            .collect(),
    }
}

#[inline(always)]
fn or_words(dst: &mut [u64], src: &[u64]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d |= *s;
    }
}

fn set_bit(words: &mut [u64], idx: usize) {
    words[idx / 64] |= 1u64 << (idx % 64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::{
        extraction::{Extractor, Search},
        report::SearchMode,
    };

    fn generated_minigrid<const N: usize, const K: usize>(
        payloads: Vec<[u8; N]>,
    ) -> GeneratedMinigrid<N, K> {
        let nodes = payloads
            .iter()
            .enumerate()
            .map(|(idx, cells)| {
                PermutationNode::from_minigrid(
                    cells,
                    PermId::try_from(idx).expect("payload index must fit into u32"),
                )
            })
            .collect();
        GeneratedMinigrid { nodes, payloads }
    }

    fn make_graph<const K: usize, const N: usize>() -> Graph<K, N> {
        Graph::new(std::array::from_fn(|_| GeneratedMinigrid {
            nodes: Vec::new(),
            payloads: Vec::new(),
        }))
    }

    fn direct_compatible_set<const N: usize, const K: usize>(
        left: &[PermutationNode<N, K>],
        right: &[PermutationNode<N, K>],
        relation: Relation,
        perm_id: usize,
    ) -> FixedBitSet {
        let mut words = vec![0u64; right.len().div_ceil(64)];
        let left_node = &left[perm_id];

        for (right_perm, right_node) in right.iter().enumerate() {
            let compatible = match relation {
                Relation::Row => left_node.check_row_compatible(right_node),
                Relation::Col => left_node.check_col_compatible(right_node),
                Relation::Not => false,
            };
            if compatible {
                set_bit(&mut words, right_perm);
            }
        }

        FixedBitSet::from_words(words, right.len())
    }

    #[test]
    fn test_examples_9x9() {
        const K: usize = 3;
        const N: usize = K * K;
        let g = make_graph::<K, N>();

        assert_eq!(g.relationship(0, 1), Relation::Row);
        assert_eq!(g.relationship(1, 2), Relation::Row);
        assert_eq!(g.relationship(3, 5), Relation::Row);

        assert_eq!(g.relationship(0, 3), Relation::Col);
        assert_eq!(g.relationship(3, 6), Relation::Col);
        assert_eq!(g.relationship(2, 8), Relation::Col);

        assert_eq!(g.relationship(0, 4), Relation::Not);
        assert_eq!(g.relationship(2, 6), Relation::Not);
        assert_eq!(g.relationship(5, 5), Relation::Not);
    }

    #[test]
    fn compatible_lookup_uses_only_related_pair_tables() {
        const K: usize = 2;
        const N: usize = 4;

        let mut graph = Graph::new([
            generated_minigrid::<N, K>(vec![[1, 2, 3, 4]]),
            generated_minigrid::<N, K>(vec![[3, 4, 1, 2], [1, 4, 2, 3]]),
            generated_minigrid::<N, K>(vec![[2, 1, 4, 3], [3, 1, 4, 2]]),
            generated_minigrid::<N, K>(vec![]),
        ]);

        graph.create_edges();

        let row_edges = graph
            .compatible_set(0, 0, 1)
            .expect("row-related minigrids must have compatibility data");
        assert!(row_edges.contains(0));
        assert!(!row_edges.contains(1));

        let reverse_row_edges = graph
            .compatible_set(1, 0, 0)
            .expect("reverse lookup must share the same pair table");
        assert!(reverse_row_edges.contains(0));

        let col_edges = graph
            .compatible_set(0, 0, 2)
            .expect("col-related minigrids must have compatibility data");
        assert!(col_edges.contains(0));
        assert!(!col_edges.contains(1));

        assert!(
            graph.compatible_set(0, 0, 3).is_none(),
            "unrelated minigrids should not allocate adjacency tables"
        );
        assert_eq!(graph.pair_count(), 4);
    }

    #[test]
    fn retain_permutations_remaps_payloads_and_edges() {
        const K: usize = 2;
        const N: usize = 4;

        let kept_cells = [1, 3, 2, 4];
        let mut graph = Graph::new([
            generated_minigrid::<N, K>(vec![[1, 2, 3, 4], kept_cells]),
            generated_minigrid::<N, K>(vec![[3, 4, 1, 2], [2, 4, 1, 3]]),
            generated_minigrid::<N, K>(vec![[2, 1, 4, 3], [3, 1, 4, 2]]),
            generated_minigrid::<N, K>(vec![[4, 3, 2, 1], [4, 2, 3, 1]]),
        ]);

        graph.create_edges();
        graph.retain_permutations(&[vec![1], vec![1], vec![1], vec![1]]);

        assert_eq!(graph.permutation_count(0), 1);
        assert_eq!(graph.permutation_cells(0, 0), &kept_cells);

        let edges = graph
            .compatible_set(0, 0, 1)
            .expect("retained related pair must still exist");
        assert_eq!(edges.len(), 1);
        assert!(edges.contains(0));

        let reverse_edges = graph
            .compatible_set(1, 0, 0)
            .expect("reverse retained pair must still exist");
        assert_eq!(reverse_edges.len(), 1);
        assert!(reverse_edges.contains(0));
    }

    #[test]
    fn signature_compression_preserves_concrete_compatibility_sets() {
        const K: usize = 2;
        const N: usize = 4;

        let generated = [
            generated_minigrid::<N, K>(vec![[1, 2, 3, 4], [2, 1, 3, 4]]),
            generated_minigrid::<N, K>(vec![[3, 4, 1, 2], [1, 2, 3, 4], [1, 4, 2, 3]]),
            generated_minigrid::<N, K>(vec![[3, 1, 4, 2], [3, 2, 4, 1], [4, 1, 3, 2]]),
            generated_minigrid::<N, K>(vec![[4, 3, 2, 1], [4, 2, 3, 1]]),
        ];
        let expected = generated.clone();
        let mut graph = Graph::new(generated);

        graph.create_edges();

        assert!(
            graph.row_indexes[0].signatures.len() < graph.minigrids[0].len(),
            "test setup should share a row signature"
        );
        assert!(
            graph.col_indexes[1].signatures.len() < graph.minigrids[1].len(),
            "test setup should share a column signature"
        );

        for perm_id in 0..expected[0].nodes.len() {
            let direct = direct_compatible_set(
                &expected[0].nodes,
                &expected[1].nodes,
                Relation::Row,
                perm_id,
            );
            let compressed = graph
                .compatible_set(0, perm_id, 1)
                .expect("row-related pair must exist");
            assert_eq!(&direct, compressed);
        }

        for perm_id in 0..expected[1].nodes.len() {
            let direct = direct_compatible_set(
                &expected[1].nodes,
                &expected[3].nodes,
                Relation::Col,
                perm_id,
            );
            let compressed = graph
                .compatible_set(1, perm_id, 3)
                .expect("column-related pair must exist");
            assert_eq!(&direct, compressed);
        }
    }

    #[test]
    fn signature_compression_preserves_search_and_extraction_results() {
        const K: usize = 2;
        const N: usize = 4;

        let mut graph = Graph::new([
            generated_minigrid::<N, K>(vec![[1, 2, 3, 4], [1, 3, 2, 4]]),
            generated_minigrid::<N, K>(vec![[3, 4, 1, 2], [2, 4, 1, 3]]),
            generated_minigrid::<N, K>(vec![[2, 1, 4, 3], [3, 1, 4, 2]]),
            generated_minigrid::<N, K>(vec![[4, 3, 2, 1], [4, 2, 3, 1]]),
        ]);
        graph.create_edges();

        let configs = Search::new(&graph)
            .with_mode(SearchMode::EnumerateAll)
            .all();
        assert_eq!(configs, vec![[0, 0, 0, 0], [1, 1, 1, 1]]);

        let solutions = Extractor::new(&graph).run_with_configurations(configs.clone());
        assert_eq!(solutions.len(), 2);
        assert!(solutions.iter().all(|solution| solution.board.is_valid()));
        assert_eq!(
            solutions
                .iter()
                .map(|solution| solution.permutation_ids)
                .collect::<Vec<_>>(),
            configs
        );
    }
}
