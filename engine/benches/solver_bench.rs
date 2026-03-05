use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dedaliano_engine::solver::{buckling, linear, modal, pdelta, plastic};
use dedaliano_engine::types::*;
use std::collections::HashMap;

// ─── Helpers ────────────────────────────────────────────────

fn make_input(
    nodes: Vec<(usize, f64, f64)>,
    mats: Vec<(usize, f64, f64)>,
    secs: Vec<(usize, f64, f64)>,
    elems: Vec<(usize, &str, usize, usize, usize, usize, bool, bool)>,
    sups: Vec<(usize, usize, &str)>,
    loads: Vec<SolverLoad>,
) -> SolverInput {
    let mut nodes_map = HashMap::new();
    for (id, x, y) in nodes {
        nodes_map.insert(id.to_string(), SolverNode { id, x, y });
    }
    let mut mats_map = HashMap::new();
    for (id, e, nu) in mats {
        mats_map.insert(id.to_string(), SolverMaterial { id, e, nu });
    }
    let mut secs_map = HashMap::new();
    for (id, a, iz) in secs {
        secs_map.insert(id.to_string(), SolverSection { id, a, iz });
    }
    let mut elems_map = HashMap::new();
    for (id, t, ni, nj, mi, si, hs, he) in elems {
        elems_map.insert(
            id.to_string(),
            SolverElement {
                id,
                elem_type: t.to_string(),
                node_i: ni,
                node_j: nj,
                material_id: mi,
                section_id: si,
                hinge_start: hs,
                hinge_end: he,
            },
        );
    }
    let mut sups_map = HashMap::new();
    for (id, nid, t) in sups {
        sups_map.insert(
            id.to_string(),
            SolverSupport {
                id,
                node_id: nid,
                support_type: t.to_string(),
                kx: None,
                ky: None,
                kz: None,
                dx: None,
                dy: None,
                drz: None,
                angle: None,
            },
        );
    }
    SolverInput {
        nodes: nodes_map,
        materials: mats_map,
        sections: secs_map,
        elements: elems_map,
        supports: sups_map,
        loads,
    }
}

/// Multi-element simply-supported beam with UDL.
fn make_ss_beam(n_elem: usize) -> SolverInput {
    let l = 10.0;
    let e = 200_000.0;
    let a = 0.01;
    let iz = 1e-4;
    let q = -10.0;
    let elem_len = l / n_elem as f64;

    let mut nodes = Vec::new();
    for i in 0..=n_elem {
        nodes.push((i + 1, i as f64 * elem_len, 0.0));
    }
    let mut elems = Vec::new();
    for i in 0..n_elem {
        elems.push((i + 1, "frame", i + 1, i + 2, 1, 1, false, false));
    }
    let sups = vec![(1, 1, "pinned"), (2, n_elem + 1, "rollerX")];
    let mut loads = Vec::new();
    for i in 0..n_elem {
        loads.push(SolverLoad::Distributed(SolverDistributedLoad {
            element_id: i + 1,
            q_i: q,
            q_j: q,
            a: None,
            b: None,
        }));
    }

    make_input(
        nodes,
        vec![(1, e, 0.3)],
        vec![(1, a, iz)],
        elems,
        sups,
        loads,
    )
}

/// Multi-story frame (n_stories × n_bays).
fn make_frame(n_stories: usize, n_bays: usize) -> SolverInput {
    let h = 3.0; // story height
    let w = 6.0; // bay width
    let e = 200_000.0;
    let a = 0.01;
    let iz = 1e-4;

    let mut nodes = Vec::new();
    let mut node_id = 1;
    // Grid: (n_bays+1) columns × (n_stories+1) rows
    for j in 0..=n_stories {
        for i in 0..=n_bays {
            nodes.push((node_id, i as f64 * w, j as f64 * h));
            node_id += 1;
        }
    }
    let cols = n_bays + 1;

    let mut elems = Vec::new();
    let mut eid = 1;
    // Columns
    for j in 0..n_stories {
        for i in 0..=n_bays {
            let ni = j * cols + i + 1;
            let nj = (j + 1) * cols + i + 1;
            elems.push((eid, "frame", ni, nj, 1, 1, false, false));
            eid += 1;
        }
    }
    // Beams
    for j in 1..=n_stories {
        for i in 0..n_bays {
            let ni = j * cols + i + 1;
            let nj = j * cols + i + 2;
            elems.push((eid, "frame", ni, nj, 1, 1, false, false));
            eid += 1;
        }
    }

    // Fixed supports at base
    let mut sups = Vec::new();
    for i in 0..=n_bays {
        sups.push((i + 1, i + 1, "fixed"));
    }

    // Lateral + gravity loads at each floor
    let mut loads = Vec::new();
    for j in 1..=n_stories {
        // Lateral at left node
        loads.push(SolverLoad::Nodal(SolverNodalLoad {
            node_id: j * cols + 1,
            fx: 10.0,
            fy: 0.0,
            mz: 0.0,
        }));
        // Gravity at each node
        for i in 0..=n_bays {
            loads.push(SolverLoad::Nodal(SolverNodalLoad {
                node_id: j * cols + i + 1,
                fx: 0.0,
                fy: -50.0,
                mz: 0.0,
            }));
        }
    }

    make_input(
        nodes,
        vec![(1, e, 0.3)],
        vec![(1, a, iz)],
        elems,
        sups,
        loads,
    )
}

/// Column for buckling analysis.
fn make_column(n_elem: usize) -> SolverInput {
    let l = 5.0;
    let e = 200_000.0;
    let a = 0.01;
    let iz = 1e-4;
    let p = -100.0;
    let elem_len = l / n_elem as f64;

    let mut nodes = Vec::new();
    for i in 0..=n_elem {
        nodes.push((i + 1, i as f64 * elem_len, 0.0));
    }
    let mut elems = Vec::new();
    for i in 0..n_elem {
        elems.push((i + 1, "frame", i + 1, i + 2, 1, 1, false, false));
    }

    make_input(
        nodes,
        vec![(1, e, 0.3)],
        vec![(1, a, iz)],
        elems,
        vec![(1, 1, "pinned"), (2, n_elem + 1, "rollerX")],
        vec![SolverLoad::Nodal(SolverNodalLoad {
            node_id: n_elem + 1,
            fx: p,
            fy: 0.0,
            mz: 0.0,
        })],
    )
}

// ─── JSON round-trip benchmark (simulates WASM boundary) ────

fn bench_json_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_roundtrip");

    for n in [4, 16, 64] {
        let input = make_ss_beam(n);
        let json = serde_json::to_string(&input).unwrap();
        group.bench_with_input(BenchmarkId::new("serialize", n), &input, |b, input| {
            b.iter(|| serde_json::to_string(input).unwrap());
        });
        group.bench_with_input(
            BenchmarkId::new("deserialize", n),
            &json,
            |b, json| {
                b.iter(|| serde_json::from_str::<SolverInput>(json).unwrap());
            },
        );
    }
    group.finish();
}

// ─── Linear solve benchmarks ─────────────────────────────────

fn bench_linear_beam(c: &mut Criterion) {
    let mut group = c.benchmark_group("linear_beam");

    for n in [4, 8, 16, 32, 64] {
        let input = make_ss_beam(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &input, |b, input| {
            b.iter(|| linear::solve_2d(input).unwrap());
        });
    }
    group.finish();
}

fn bench_linear_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("linear_frame");

    for &(stories, bays) in &[(3, 2), (5, 3), (10, 4), (20, 5)] {
        let input = make_frame(stories, bays);
        let label = format!("{}s_{}b", stories, bays);
        group.bench_with_input(BenchmarkId::new("solve", &label), &input, |b, input| {
            b.iter(|| linear::solve_2d(input).unwrap());
        });
    }
    group.finish();
}

// ─── Full WASM-like JSON solve ───────────────────────────────

fn bench_json_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_solve_2d");

    for &(stories, bays) in &[(3, 2), (10, 4)] {
        let input = make_frame(stories, bays);
        let json = serde_json::to_string(&input).unwrap();
        let label = format!("{}s_{}b", stories, bays);
        group.bench_with_input(BenchmarkId::new("full", &label), &json, |b, json| {
            b.iter(|| {
                let input: SolverInput = serde_json::from_str(json).unwrap();
                let result = linear::solve_2d(&input).unwrap();
                serde_json::to_string(&result).unwrap()
            });
        });
    }
    group.finish();
}

// ─── Buckling benchmarks ─────────────────────────────────────

fn bench_buckling(c: &mut Criterion) {
    let mut group = c.benchmark_group("buckling");

    for n in [4, 8, 16] {
        let input = make_column(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &input, |b, input| {
            b.iter(|| buckling::solve_buckling_2d(input, 4).unwrap());
        });
    }
    group.finish();
}

// ─── Modal benchmarks ────────────────────────────────────────

fn bench_modal(c: &mut Criterion) {
    let mut group = c.benchmark_group("modal");

    let density = 7850.0;
    for n in [4, 8, 16] {
        let input = make_ss_beam(n);
        let mut densities = HashMap::new();
        densities.insert("1".to_string(), density);

        group.bench_with_input(BenchmarkId::from_parameter(n), &(input, densities), |b, (input, dens)| {
            b.iter(|| modal::solve_modal_2d(input, dens, 4).unwrap());
        });
    }
    group.finish();
}

// ─── P-Delta benchmarks ──────────────────────────────────────

fn bench_pdelta(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdelta");

    for &(stories, bays) in &[(3, 2), (5, 3)] {
        let input = make_frame(stories, bays);
        let label = format!("{}s_{}b", stories, bays);
        group.bench_with_input(BenchmarkId::new("solve", &label), &input, |b, input| {
            b.iter(|| pdelta::solve_pdelta_2d(input, 20, 1e-4).unwrap());
        });
    }
    group.finish();
}

// ─── Plastic benchmarks ──────────────────────────────────────

fn bench_plastic(c: &mut Criterion) {
    let mut group = c.benchmark_group("plastic");

    let n_elem = 4;
    let a_area = 0.01;
    let iz = 1e-4;
    let fy = 250.0;
    let b_width = 0.1;
    let h_depth = 0.2;

    let solver = make_ss_beam(n_elem);
    let mut sections = HashMap::new();
    for i in 0..n_elem {
        sections.insert(
            (i + 1).to_string(),
            PlasticSectionData {
                a: a_area,
                iz,
                material_id: 1,
                b: Some(b_width),
                h: Some(h_depth),
            },
        );
    }
    let mut materials = HashMap::new();
    materials.insert(
        "1".to_string(),
        PlasticMaterialData { fy: Some(fy) },
    );
    let input = PlasticInput {
        solver,
        sections,
        materials,
        max_hinges: None,
        mp_overrides: None,
    };

    group.bench_function("ss_beam_4elem", |b| {
        b.iter(|| plastic::solve_plastic_2d(&input).unwrap());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_json_roundtrip,
    bench_linear_beam,
    bench_linear_frame,
    bench_json_solve,
    bench_buckling,
    bench_modal,
    bench_pdelta,
    bench_plastic,
);
criterion_main!(benches);
