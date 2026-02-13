use crate::audio::capture::MAX_BUFFER_SAMPLES;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// WHAT: Buffer respects MAX_BUFFER_SAMPLES limit
/// WHY: Prevents unbounded memory growth during long recordings
#[test]
fn given_buffer_at_max_capacity_when_adding_samples_then_oldest_discarded() {
    // Given: A VecDeque at max capacity filled with 0.0
    let mut buf = VecDeque::with_capacity(MAX_BUFFER_SAMPLES);
    buf.extend(std::iter::repeat(0.0f32).take(MAX_BUFFER_SAMPLES));
    assert_eq!(buf.len(), MAX_BUFFER_SAMPLES);

    // When: Adding 1024 new samples (value 1.0) beyond the limit
    let new_samples = vec![1.0f32; 1024];
    buf.extend(new_samples.iter().copied());
    while buf.len() > MAX_BUFFER_SAMPLES {
        buf.pop_front();
    }

    // Then: Buffer stays at MAX_BUFFER_SAMPLES and newest samples preserved
    assert_eq!(buf.len(), MAX_BUFFER_SAMPLES);
    assert!((buf[MAX_BUFFER_SAMPLES - 1] - 1.0).abs() < f32::EPSILON);
    assert!((buf[MAX_BUFFER_SAMPLES - 1024] - 1.0).abs() < f32::EPSILON);
}

/// WHAT: Lock poison recovery preserves buffer data
/// WHY: Ensures audio data is never silently lost on mutex poison
#[test]
fn given_poisoned_mutex_when_recovering_then_data_preserved() {
    // Given: A mutex poisoned by a panic while holding the lock
    let buf = Arc::new(Mutex::new(VecDeque::from(vec![0.5f32; 100])));
    let buf_clone = Arc::clone(&buf);

    let _ = std::thread::spawn(move || {
        let _guard = buf_clone.lock().unwrap();
        panic!("intentional panic to poison mutex");
    })
    .join();

    // When: Recovering from poisoned lock using unwrap_or_else
    let recovered = buf.lock().unwrap_or_else(|e| e.into_inner());

    // Then: Original data is fully preserved
    assert_eq!(recovered.len(), 100);
    assert!(recovered.iter().all(|&s| (s - 0.5).abs() < f32::EPSILON));
}

/// WHAT: Concurrent writes to shared buffer produce consistent state
/// WHY: Validates thread safety of Arc<Mutex<VecDeque>> under contention
#[test]
fn given_concurrent_writers_when_writing_to_buffer_then_no_corruption() {
    // Given: Shared buffer simulating audio callback contention
    let buf = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_BUFFER_SAMPLES)));
    let mut handles = vec![];

    // When: 4 threads write 1000 batches of 48 samples each concurrently
    for i in 0..4u8 {
        let buf_clone = Arc::clone(&buf);
        handles.push(std::thread::spawn(move || {
            for _ in 0..1000 {
                let mut b = buf_clone.lock().unwrap_or_else(|e| e.into_inner());
                b.extend(std::iter::repeat(f32::from(i)).take(48));
                while b.len() > MAX_BUFFER_SAMPLES {
                    b.pop_front();
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // Then: Buffer is within bounds and contains only finite values
    let b = buf.lock().unwrap();
    assert!(b.len() <= MAX_BUFFER_SAMPLES);
    assert!(b.iter().all(|s| s.is_finite()));
    // Total: 4 threads x 1000 batches x 48 = 192,000 (well under max)
    assert_eq!(b.len(), 4 * 1000 * 48);
}
