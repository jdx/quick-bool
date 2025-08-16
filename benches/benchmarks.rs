use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quick_bool::QuickBool;
use std::sync::LazyLock;
use std::sync::OnceLock;

// Define a simple computation function for benchmarking
fn expensive_computation() -> bool {
    // Simulate some expensive work
    let mut result = 0;
    for i in 0..1000 {
        result += i * i;
    }
    result % 2 == 0
}

// Benchmark QuickBool
fn benchmark_quick_bool(c: &mut Criterion) {
    let mut group = c.benchmark_group("QuickBool");
    
    // Benchmark first access (computation)
    group.bench_function("first_access", |b| {
        b.iter(|| {
            let qb = QuickBool::new();
            black_box(qb.get_or_set(expensive_computation));
        });
    });
    
    // Benchmark subsequent access (cached)
    group.bench_function("cached_access", |b| {
        let qb = QuickBool::new();
        // Prime the cache
        qb.get_or_set(expensive_computation);
        
        b.iter(|| {
            black_box(qb.get_or_set(|| panic!("Should not execute")));
        });
    });

    // Benchmark concurrent access
    group.bench_function("concurrent_access", |b| {
        b.iter(|| {
            use std::sync::Arc;
            use std::thread;
            
            let qb = Arc::new(QuickBool::new());
            let mut handles = vec![];
            
            // Spawn 4 threads that all try to access simultaneously
            for _ in 0..4 {
                let qb_clone = Arc::clone(&qb);
                let handle = thread::spawn(move || {
                    qb_clone.get_or_set(expensive_computation)
                });
                handles.push(handle);
            }
            
            // Wait for all threads
            let results: Vec<bool> = handles.into_iter()
                .map(|h| h.join().unwrap())
                .collect();
            
            black_box(results);
        });
    });
    
    group.finish();
}

// Benchmark LazyLock (from std)
fn benchmark_lazy_lock(c: &mut Criterion) {
    let mut group = c.benchmark_group("LazyLock");
    
    // Benchmark first access (computation)
    group.bench_function("first_access", |b| {
        b.iter(|| {
            let lazy_bool = LazyLock::new(expensive_computation);
            black_box(*lazy_bool);
        });
    });
    
    // Benchmark subsequent access (cached)
    group.bench_function("cached_access", |b| {
        let lazy_bool = LazyLock::new(expensive_computation);
        // Prime the cache
        black_box(*lazy_bool);
        
        b.iter(|| {
            black_box(*lazy_bool);
        });
    });
    
    // Benchmark concurrent access
    group.bench_function("concurrent_access", |b| {
        b.iter(|| {
            use std::sync::Arc;
            use std::thread;
            
            let lazy_bool = Arc::new(LazyLock::new(expensive_computation));
            let mut handles = vec![];
            
            // Spawn 4 threads that all try to access simultaneously
            for _ in 0..4 {
                let lazy_clone = Arc::clone(&lazy_bool);
                let handle = thread::spawn(move || {
                    // Dereference to get the bool value
                    **lazy_clone
                });
                handles.push(handle);
            }
            
            // Wait for all threads
            let results: Vec<bool> = handles.into_iter()
                .map(|h| h.join().unwrap())
                .collect();
            
            black_box(results);
        });
    });
    
    group.finish();
}

// Benchmark OnceLock for comparison
fn benchmark_once_lock(c: &mut Criterion) {
    let mut group = c.benchmark_group("OnceLock");
    
    // Benchmark first access (computation)
    group.bench_function("first_access", |b| {
        b.iter(|| {
            let once_bool = OnceLock::new();
            black_box(once_bool.get_or_init(expensive_computation));
        });
    });
    
    // Benchmark subsequent access (cached)
    group.bench_function("cached_access", |b| {
        let once_bool = OnceLock::new();
        // Prime the cache
        once_bool.get_or_init(expensive_computation);
        
        b.iter(|| {
            black_box(once_bool.get().unwrap());
        });
    });
    
    // Benchmark concurrent access
    group.bench_function("concurrent_access", |b| {
        b.iter(|| {
            use std::sync::Arc;
            use std::thread;
            
            let once_bool = Arc::new(OnceLock::new());
            let mut handles = vec![];
            
            // Spawn 4 threads that all try to access simultaneously
            for _ in 0..4 {
                let once_clone = Arc::clone(&once_bool);
                let handle = thread::spawn(move || {
                    // Get the value and clone it to avoid lifetime issues
                    let value = *once_clone.get_or_init(expensive_computation);
                    black_box(value)
                });
                handles.push(handle);
            }
            
            // Wait for all threads
            let results: Vec<bool> = handles.into_iter()
                .map(|h| h.join().unwrap())
                .collect();
            
            black_box(results);
        });
    });
    
    group.finish();
}

// Benchmark comparison between all implementations
fn benchmark_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("Comparison");
    
    // Compare first access performance
    group.bench_function("QuickBool_first", |b| {
        b.iter(|| {
            let qb = QuickBool::new();
            black_box(qb.get_or_set(expensive_computation));
        });
    });
    
    group.bench_function("LazyLock_first", |b| {
        b.iter(|| {
            let lazy_bool = LazyLock::new(expensive_computation);
            black_box(*lazy_bool);
        });
    });
    
    group.bench_function("OnceLock_first", |b| {
        b.iter(|| {
            let once_bool = OnceLock::new();
            black_box(once_bool.get_or_init(expensive_computation));
        });
    });
    
    // Compare cached access performance
    group.bench_function("QuickBool_cached", |b| {
        let qb = QuickBool::new();
        qb.get_or_set(expensive_computation);
        
        b.iter(|| {
            black_box(qb.get_or_set(|| panic!("Should not execute")));
        });
    });
    
    group.bench_function("LazyLock_cached", |b| {
        let lazy_bool = LazyLock::new(expensive_computation);
        black_box(*lazy_bool); // Prime the cache
        
        b.iter(|| {
            black_box(*lazy_bool);
        });
    });
    
    group.bench_function("OnceLock_cached", |b| {
        let once_bool = OnceLock::new();
        once_bool.get_or_init(expensive_computation);
        
        b.iter(|| {
            black_box(once_bool.get().unwrap());
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_quick_bool,
    benchmark_lazy_lock,
    benchmark_once_lock,
    benchmark_comparison
);
criterion_main!(benches);
