use std::sync::atomic::{AtomicU8, Ordering};

/// A lock-free boolean implementation using atomic operations.
/// 
/// This type represents a 3-way boolean state:
/// - `Unset`: The value hasn't been evaluated yet
/// - `True`: The value is true
/// - `False`: The value is false
/// 
/// Once set to true or false, the value cannot be changed, making it
/// effectively immutable after initialization.
#[derive(Debug)]
pub struct QuickBool {
    state: AtomicU8,
}

impl QuickBool {
    /// Represents the unset state
    const UNSET: u8 = 0;
    /// Represents the false state
    const FALSE: u8 = 0xFF; // Use 0xFF for better branch prediction
    /// Represents the true state
    const TRUE: u8 = 1;      // Use 1 for true (common case)

    /// Creates a new `QuickBool` in the unset state.
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(Self::UNSET),
        }
    }

    /// Gets the current value, evaluating the closure if the value is unset.
    /// 
    /// This method is lock-free and will only evaluate the closure once.
    /// Subsequent calls will return the cached value.
    /// 
    /// # Arguments
    /// 
    /// * `f` - A closure that returns a boolean value
    /// 
    /// # Returns
    /// 
    /// The boolean value, either from cache or newly computed
    /// 
    /// # Example
    /// 
    /// ```
    /// use quick_bool::QuickBool;
    /// 
    /// let quick_bool = QuickBool::new();
    /// let value = quick_bool.get_or_set(|| {
    ///     // This expensive computation only happens once
    ///     std::thread::sleep(std::time::Duration::from_millis(100));
    ///     true
    /// });
    /// 
    /// // Second call returns immediately without computation
    /// let cached_value = quick_bool.get_or_set(|| panic!("This won't execute"));
    /// assert_eq!(value, cached_value);
    /// ```
    pub fn get_or_set<F>(&self, f: F) -> bool
    where
        F: FnOnce() -> bool,
    {
        // Fast path: try relaxed load first for better performance
        let current = self.state.load(Ordering::Relaxed);
        
        // Optimize match order: TRUE first (most common), then FALSE, then UNSET
        match current {
            Self::TRUE => true,
            Self::FALSE => false,
            Self::UNSET => {
                // Value is unset, we need to compute it
                let computed_value = f();
                let target_state = if computed_value { Self::TRUE } else { Self::FALSE };
                
                // Try to set the value atomically
                match self.state.compare_exchange(
                    Self::UNSET,
                    target_state,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => computed_value,
                    Err(actual) => {
                        // Another thread set the value while we were computing
                        // Return the value that was set
                        actual == Self::TRUE
                    }
                }
            }
            _ => unreachable!("Invalid state value"),
        }
    }









    /// Gets the current value without computing it.
    /// 
    /// Returns `None` if the value is unset, `Some(true)` if true,
    /// or `Some(false)` if false.
    /// 
    /// # Example
    /// 
    /// ```
    /// use quick_bool::QuickBool;
    /// 
    /// let quick_bool = QuickBool::new();
    /// assert_eq!(quick_bool.get(), None);
    /// 
    /// quick_bool.get_or_set(|| true);
    /// assert_eq!(quick_bool.get(), Some(true));
    /// ```
    #[inline]
    pub fn get(&self) -> Option<bool> {
        // Use relaxed ordering since we're only reading
        match self.state.load(Ordering::Relaxed) {
            Self::TRUE => Some(true),
            Self::FALSE => Some(false),
            Self::UNSET => None,
            _ => unreachable!("Invalid state value"),
        }
    }

    /// Checks if the value has been set.
    /// 
    /// Returns `true` if the value is either true or false,
    /// `false` if it's still unset.
    /// 
    /// # Example
    /// 
    /// ```
    /// use quick_bool::QuickBool;
    /// 
    /// let quick_bool = QuickBool::new();
    /// assert!(!quick_bool.is_set());
    /// 
    /// quick_bool.get_or_set(|| true);
    /// assert!(quick_bool.is_set());
    /// ```
    #[inline]
    pub fn is_set(&self) -> bool {
        // Use relaxed ordering since we're only checking state
        self.state.load(Ordering::Relaxed) != Self::UNSET
    }

    /// Resets the value back to unset state.
    /// 
    /// This allows the value to be recomputed on the next call to `get_or_set`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use quick_bool::QuickBool;
    /// 
    /// let quick_bool = QuickBool::new();
    /// quick_bool.get_or_set(|| true);
    /// assert!(quick_bool.is_set());
    /// 
    /// quick_bool.reset();
    /// assert!(!quick_bool.is_set());
    /// ```
    pub fn reset(&self) {
        // Use Release ordering since we're modifying state
        self.state.store(Self::UNSET, Ordering::Release);
    }
}

impl Default for QuickBool {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for QuickBool {
    fn clone(&self) -> Self {
        Self {
            state: AtomicU8::new(self.state.load(Ordering::Relaxed)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_new_quick_bool() {
        let qb = QuickBool::new();
        assert_eq!(qb.get(), None);
        assert!(!qb.is_set());
    }

    #[test]
    fn test_get_or_set_once() {
        let qb = QuickBool::new();
        let value = qb.get_or_set(|| true);
        assert!(value);
        assert_eq!(qb.get(), Some(true));
        assert!(qb.is_set());
    }

    #[test]
    fn test_get_or_set_multiple_calls() {
        let qb = QuickBool::new();
        let mut call_count = 0;
        
        let value1 = qb.get_or_set(|| {
            call_count += 1;
            false
        });
        
        let value2 = qb.get_or_set(|| {
            call_count += 1;
            panic!("This should not execute");
        });
        
        assert!(!value1);
        assert!(!value2);
        assert_eq!(call_count, 1);
        assert_eq!(qb.get(), Some(false));
    }

    #[test]
    fn test_reset() {
        let qb = QuickBool::new();
        qb.get_or_set(|| true);
        assert!(qb.is_set());
        
        qb.reset();
        assert!(!qb.is_set());
        assert_eq!(qb.get(), None);
    }

    #[test]
    fn test_concurrent_access() {
        let qb = Arc::new(QuickBool::new());
        let mut handles = vec![];
        
        // Spawn multiple threads that all try to set the value
        for _ in 0..10 {
            let qb_clone = Arc::clone(&qb);
            let handle = thread::spawn(move || {
                qb_clone.get_or_set(|| {
                    // Simulate some computation
                    thread::sleep(std::time::Duration::from_millis(1));
                    true
                })
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        let results: Vec<bool> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        // All threads should get the same result
        assert!(results.iter().all(|&x| x));
        assert_eq!(qb.get(), Some(true));
    }

    #[test]
    fn test_clone() {
        let qb1 = QuickBool::new();
        qb1.get_or_set(|| true);
        
        let qb2 = qb1.clone();
        assert_eq!(qb2.get(), Some(true));
        assert!(qb2.is_set());
    }
}
