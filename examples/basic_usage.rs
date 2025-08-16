use quick_bool::QuickBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("QuickBool Basic Usage Example\n");
    
    // Create a new QuickBool
    let qb = QuickBool::new();
    
    // Check initial state
    println!("Initial state:");
    println!("  is_set: {}", qb.is_set());
    println!("  get(): {:?}", qb.get());
    
    // First access - this will compute the value
    println!("\nFirst access (computing value):");
    let start = std::time::Instant::now();
    let value = qb.get_or_set(|| {
        // Simulate expensive computation
        thread::sleep(Duration::from_millis(100));
        println!("    Computing expensive value...");
        true
    });
    let duration = start.elapsed();
    println!("  Result: {}", value);
    println!("  Time taken: {:?}", duration);
    
    // Check state after computation
    println!("\nAfter computation:");
    println!("  is_set: {}", qb.is_set());
    println!("  get(): {:?}", qb.get());
    
    // Second access - this returns immediately
    println!("\nSecond access (cached):");
    let start = std::time::Instant::now();
    let cached_value = qb.get_or_set(|| {
        panic!("This should never execute!");
    });
    let duration = start.elapsed();
    println!("  Result: {}", cached_value);
    println!("  Time taken: {:?}", duration);
    
    // Demonstrate reset functionality
    println!("\nResetting QuickBool:");
    qb.reset();
    println!("  is_set: {}", qb.is_set());
    println!("  get(): {:?}", qb.get());
    
    // Can compute again after reset
    println!("\nRecomputing after reset:");
    let value2 = qb.get_or_set(|| {
        println!("    Computing new value...");
        false
    });
    println!("  New result: {}", value2);
    
    // Demonstrate thread safety
    println!("\nThread safety demonstration:");
    let qb_arc = Arc::new(QuickBool::new());
    let mut handles = vec![];
    
    // Spawn multiple threads
    for i in 0..3 {
        let qb_clone = Arc::clone(&qb_arc);
        let handle = thread::spawn(move || {
            let thread_value = qb_clone.get_or_set(|| {
                println!("    Thread {} computing value...", i);
                thread::sleep(Duration::from_millis(50));
                i % 2 == 0
            });
            println!("    Thread {} got value: {}", i, thread_value);
            thread_value
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    let results: Vec<bool> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    println!("  All threads completed. Results: {:?}", results);
    println!("  Final value: {:?}", qb_arc.get());
    
    println!("\nExample completed successfully!");
}
