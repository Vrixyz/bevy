use bevy_ecs::{
    component::Component,
    entity::Entity,
    prelude::{Added, Changed},
    world::World,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{prelude::SliceRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;

criterion_group!(
    benches,
    all_added_detection,
    all_changed_detection,
    few_changed_detection,
);
criterion_main!(benches);

#[derive(Component, Default)]
#[component(storage = "Table")]
struct Table;
#[derive(Component, Default)]
#[component(storage = "SparseSet")]
struct Sparse;

const RANGE_ENTITIES_TO_BENCH_COUNT: std::ops::Range<u32> = 5..7;

type BenchGroup<'a> = criterion::BenchmarkGroup<'a, criterion::measurement::WallTime>;

fn deterministic_rand() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}

fn setup<T: Component + Default>(entity_count: u32) -> World {
    let mut world = World::default();
    world.spawn_batch((0..entity_count).map(|_| (T::default(),)));
    black_box(world)
}

fn generic_bench<P: Copy>(
    bench_group: &mut BenchGroup,
    mut benches: Vec<Box<dyn FnMut(&mut BenchGroup, P)>>,
    bench_parameters: P,
) {
    for b in &mut benches {
        b(bench_group, bench_parameters);
    }
}

fn all_added_detection_generic<T: Component + Default>(group: &mut BenchGroup, entity_count: u32) {
    group.bench_function(
        format!("{}_entities_{}", entity_count, std::any::type_name::<T>()),
        |bencher| {
            bencher.iter_batched(
                || setup::<T>(entity_count),
                |mut world| {
                    let mut count = 0;
                    let mut query = world.query_filtered::<Entity, Added<T>>();
                    for entity in query.iter(&world) {
                        black_box(entity);
                        count += 1;
                    }
                    assert_eq!(entity_count, count);
                },
                criterion::BatchSize::LargeInput,
            );
        },
    );
}

fn all_added_detection(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("all_added_detection");
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));
    for entity_count in RANGE_ENTITIES_TO_BENCH_COUNT.map(|i| i * 10_000) {
        generic_bench(
            &mut group,
            vec![
                Box::new(all_added_detection_generic::<Table>),
                Box::new(all_added_detection_generic::<Sparse>),
            ],
            entity_count,
        );
    }
}

fn all_changed_detection_generic<T: Component + Default>(
    group: &mut BenchGroup,
    entity_count: u32,
) {
    group.bench_function(
        format!("{}_entities_{}", entity_count, std::any::type_name::<T>()),
        |bencher| {
            bencher.iter_batched(
                || {
                    let mut world = setup::<Table>(entity_count);
                    world.clear_trackers();
                    let mut query = world.query::<&mut Table>();
                    for mut table in query.iter_mut(&mut world) {
                        black_box(&mut *table);
                    }
                    world
                },
                |mut world| {
                    let mut count = 0;
                    let mut query = world.query_filtered::<Entity, Changed<Table>>();
                    for entity in query.iter(&world) {
                        black_box(entity);
                        count += 1;
                    }
                    assert_eq!(entity_count, count);
                },
                criterion::BatchSize::LargeInput,
            );
        },
    );
}

fn all_changed_detection(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("all_changed_detection");
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));
    for entity_count in RANGE_ENTITIES_TO_BENCH_COUNT.map(|i| i * 10_000) {
        generic_bench(
            &mut group,
            vec![
                Box::new(all_changed_detection_generic::<Table>),
                Box::new(all_changed_detection_generic::<Sparse>),
            ],
            entity_count,
        );
    }
}

fn few_changed_detection_generic<T: Component + Default>(
    group: &mut BenchGroup,
    entity_count: u32,
) {
    let ratio_to_modify = 0.1;
    let amount_to_modify = (entity_count as f32 * ratio_to_modify) as usize;
    group.bench_function(
        format!("{}_entities_{}", entity_count, std::any::type_name::<T>()),
        |bencher| {
            bencher.iter_batched(
                || {
                    let mut world = setup::<Table>(entity_count);
                    world.clear_trackers();
                    let mut query = world.query::<&mut Table>();
                    let mut to_modify: Vec<bevy_ecs::prelude::Mut<Table>> =
                        query.iter_mut(&mut world).collect();
                    to_modify.shuffle(&mut deterministic_rand());
                    for table in to_modify[0..amount_to_modify].iter_mut() {
                        black_box(&mut *table);
                    }
                    world
                },
                |mut world| {
                    let mut query = world.query_filtered::<Entity, Changed<Table>>();
                    for entity in query.iter(&world) {
                        black_box(entity);
                    }
                },
                criterion::BatchSize::LargeInput,
            );
        },
    );
}

fn few_changed_detection(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("few_changed_detection");
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));
    for entity_count in RANGE_ENTITIES_TO_BENCH_COUNT.map(|i| i * 10_000) {
        generic_bench(
            &mut group,
            vec![
                Box::new(few_changed_detection_generic::<Table>),
                Box::new(few_changed_detection_generic::<Sparse>),
            ],
            entity_count,
        );
    }
}
