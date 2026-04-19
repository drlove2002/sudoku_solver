use criterion::{Criterion, criterion_group, criterion_main};
use solver::types::graph::{Graph, PermutationNode, Relation};
use std::hint::black_box;

/// Branch-free implementation (current production code)
#[inline]
fn relationship_branchfree<const K: usize>(a: usize, b: usize) -> Relation {
    let row_eq = (((a / K) ^ (b / K)) == 0) as usize;
    let col_eq = (((a % K) ^ (b % K)) == 0) as usize;
    let mask = row_eq | (col_eq << 1);
    Relation::from_mask(mask)
}

/// Conditional branching implementation (baseline for comparison)
#[inline]
fn relationship_branching<const K: usize>(a: usize, b: usize) -> Relation {
    if a / K == b / K {
        if a % K == b % K {
            Relation::Not
        } else {
            Relation::Row
        }
    } else if a % K == b % K {
        Relation::Col
    } else {
        Relation::Not
    }
}

/// Benchmark random minigrid pairs (uniform distribution)
fn bench_random_pairs(c: &mut Criterion) {
    const K: usize = 3;
    const N: usize = 9;

    let pairs: Vec<(usize, usize)> = (0..1000).map(|i| ((i * 7) % N, (i * 13) % N)).collect();

    let mut group = c.benchmark_group("random_pairs");

    group.bench_function("branchfree", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branchfree::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.bench_function("branching", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branching::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.finish();
}

/// Benchmark sequential pairs (0,1), (1,2), (2,3)...
fn bench_sequential_pairs(c: &mut Criterion) {
    const K: usize = 3;
    const N: usize = 9;

    let pairs: Vec<(usize, usize)> = (0..N - 1).map(|i| (i, i + 1)).collect();

    let mut group = c.benchmark_group("sequential_pairs");

    group.bench_function("branchfree", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branchfree::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.bench_function("branching", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branching::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.finish();
}

/// Benchmark same-row pairs only
fn bench_same_row_pairs(c: &mut Criterion) {
    const K: usize = 3;

    let pairs: Vec<(usize, usize)> = vec![
        (0, 1),
        (0, 2),
        (1, 2), // Row 0
        (3, 4),
        (3, 5),
        (4, 5), // Row 1
        (6, 7),
        (6, 8),
        (7, 8), // Row 2
    ];

    let mut group = c.benchmark_group("same_row_pairs");

    group.bench_function("branchfree", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branchfree::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.bench_function("branching", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branching::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.finish();
}

/// Benchmark same-column pairs only
fn bench_same_col_pairs(c: &mut Criterion) {
    const K: usize = 3;

    let pairs: Vec<(usize, usize)> = vec![
        (0, 3),
        (0, 6),
        (3, 6), // Col 0
        (1, 4),
        (1, 7),
        (4, 7), // Col 1
        (2, 5),
        (2, 8),
        (5, 8), // Col 2
    ];

    let mut group = c.benchmark_group("same_col_pairs");

    group.bench_function("branchfree", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branchfree::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.bench_function("branching", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branching::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.finish();
}

/// Benchmark worst-case: alternating same-block and different-block
fn bench_alternating_patterns(c: &mut Criterion) {
    const K: usize = 3;

    let pairs: Vec<(usize, usize)> = vec![
        (0, 0),
        (0, 4),
        (0, 8), // Same, different, different
        (1, 1),
        (1, 3),
        (1, 5), // Same, different, different
        (2, 2),
        (2, 6),
        (2, 7), // Same, different, different
    ];

    let mut group = c.benchmark_group("alternating_patterns");

    group.bench_function("branchfree", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branchfree::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.bench_function("branching", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(relationship_branching::<K>(black_box(a), black_box(b)));
            }
        })
    });

    group.finish();
}

/// Benchmark single call latency (isolates function overhead)
fn bench_single_call(c: &mut Criterion) {
    const K: usize = 3;

    let mut group = c.benchmark_group("single_call");

    group.bench_function("branchfree", |b| {
        b.iter(|| black_box(relationship_branchfree::<K>(black_box(2), black_box(5))))
    });

    group.bench_function("branching", |b| {
        b.iter(|| black_box(relationship_branching::<K>(black_box(2), black_box(5))))
    });

    group.finish();
}

/// Benchmark using actual Graph API (matches production usage)
fn bench_graph_api(c: &mut Criterion) {
    const K: usize = 3;
    const N: usize = 9;

    // Create minimal graph for testing
    let graph: Graph<K, N> = Graph::new([const { Vec::<PermutationNode<N, K>>::new() }; N]);

    let pairs: Vec<(usize, usize)> = (0..1000).map(|i| ((i * 7) % N, (i * 13) % N)).collect();

    c.bench_function("graph_api", |b| {
        b.iter(|| {
            for &(a, b) in &pairs {
                black_box(graph.relationship(black_box(a), black_box(b)));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_random_pairs,
    bench_sequential_pairs,
    bench_same_row_pairs,
    bench_same_col_pairs,
    bench_alternating_patterns,
    bench_single_call,
    bench_graph_api
);
criterion_main!(benches);
