use quick_bool::QuickBool;
use std::sync::LazyLock;
use std::time::Instant;

// Simulate expensive computation
fn expensive_computation() -> bool {
    // Simulate some work
    let mut result: u64 = 0;
    for i in 0..10_000 {
        result = result.wrapping_add((i * i) as u64);
    }
    result % 2 == 0
}

fn main() {
    println!("QuickBool vs LazyLock Performance Comparison\n");
    
    // Test QuickBool
    println!("Testing QuickBool:");
    let qb = QuickBool::new();
    
    // First access (computation)
    let start = Instant::now();
    let qb_value = qb.get_or_set(expensive_computation);
    let qb_first_time = start.elapsed();
    println!("  First access: {:?} (result: {})", qb_first_time, qb_value);
    
    // Cached access
    let start = Instant::now();
    let _qb_cached = qb.get_or_set(|| panic!("Should not execute"));
    let qb_cached_time = start.elapsed();
    println!("  Cached access: {:?}", qb_cached_time);
    
    // Test LazyLock
    println!("\nTesting LazyLock:");
    let lazy_bool = LazyLock::new(expensive_computation);
    
    // First access (computation)
    let start = Instant::now();
    let lazy_value = *lazy_bool;
    let lazy_first_time = start.elapsed();
    println!("  First access: {:?} (result: {})", lazy_first_time, lazy_value);
    
    // Cached access
    let start = Instant::now();
    let _lazy_cached = *lazy_bool;
    let lazy_cached_time = start.elapsed();
    println!("  Cached access: {:?}", lazy_cached_time);
    
    // Performance summary
    println!("\nPerformance Summary:");
    println!("  First access:");
    println!("    QuickBool: {:?}", qb_first_time);
    println!("    LazyLock:  {:?}", lazy_first_time);
    
    let first_ratio = qb_first_time.as_nanos() as f64 / lazy_first_time.as_nanos() as f64;
    if first_ratio > 1.0 {
        println!("    LazyLock is {:.2}x faster for first access", first_ratio);
    } else {
        println!("    QuickBool is {:.2}x faster for first access", 1.0 / first_ratio);
    }
    
    println!("\n  Cached access:");
    println!("    QuickBool: {:?}", qb_cached_time);
    println!("    LazyLock:  {:?}", lazy_cached_time);
    
    let cached_ratio = qb_cached_time.as_nanos() as f64 / lazy_cached_time.as_nanos() as f64;
    if cached_ratio > 1.0 {
        println!("    LazyLock is {:.2}x faster for cached access", cached_ratio);
    } else {
        println!("    QuickBool is {:.2}x faster for cached access", 1.0 / cached_ratio);
    }
    
    // Demonstrate QuickBool's reset capability
    println!("\nQuickBool Reset Capability:");
    qb.reset();
    println!("  After reset - is_set: {}", qb.is_set());
    
    let start = Instant::now();
    let qb_new_value = qb.get_or_set(|| !qb_value);
    let qb_reset_time = start.elapsed();
    println!("  Recomputed value: {} (took {:?})", qb_new_value, qb_reset_time);
    
    println!("\nNote: LazyLock cannot be reset once initialized.");
}
