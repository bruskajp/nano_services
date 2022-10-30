//use mycrate::fibonacci;

mod channel_options;

use std::thread;
use setup_raw::{ThingyWorker, ThingyController};
use std::sync::{Arc, Mutex};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[inline]
fn multi_thread_tester(num_threads: i32, num_events: i32, thingy: &ThingyController, method: fn(&ThingyController, i32) -> i32) {
  let mut handles = Vec::new();

  for i in 1..num_threads {
    let thingy_clone = thingy.controller_copy();
    let i_clone = i.clone();
    handles.push(thread::spawn(move || {
      for _ in 1..num_events {
        method(&thingy_clone, i_clone);
      }
    }));
  }

  for handle in handles {
    handle.join(); // Wait for the threads to finish
  }
}


// Results
//  - unsafe is fast, but not safe (it is not safe across threads)
//  - new_channel is twice as slow, but is safe
//  - futures_oneshot_channel is faster than the unsafe version
//  - tokio_oneshot_channel is slightly slower than the futures_oneshot_channel, but it is safe to use with tokio
//    - This should not be pertinent because each class is already its own thread.
//    - If I were to use this, It would be an overhaul such that classes could have async methods, which would then need the tokio scheduler
pub fn channel_options_benchmark(c: &mut Criterion) {
  let counter = Arc::new(Mutex::new(-1));
  let (thingy_handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  
  c.bench_function("set get unsafe 1", |b| b.iter(|| 
    multi_thread_tester(black_box(1), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_unsafe))));
  c.bench_function("set get new channel 1", |b| b.iter(|| 
    multi_thread_tester(black_box(1), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_channel))));
  c.bench_function("set get new oneshot channel futures 1", |b| b.iter(|| 
    multi_thread_tester(black_box(1), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_oneshot_channel_futures))));
  c.bench_function("set get new oneshot channel tokio 1", |b| b.iter(|| 
    multi_thread_tester(black_box(1), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_oneshot_channel_tokio))));

  c.bench_function("set get unsafe 2", |b| b.iter(|| 
    multi_thread_tester(black_box(2), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_unsafe))));
  c.bench_function("set get new channel 2", |b| b.iter(||
    multi_thread_tester(black_box(2), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_channel))));
  c.bench_function("set get new oneshot channel futures 2", |b| b.iter(||
    multi_thread_tester(black_box(2), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_oneshot_channel_futures))));
  c.bench_function("set get new oneshot channel tokio 2", |b| b.iter(||
    multi_thread_tester(black_box(2), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_oneshot_channel_tokio))));

  c.bench_function("set get unsafe 10", |b| b.iter(||
    multi_thread_tester(black_box(10), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_unsafe))));
  c.bench_function("set get new channel 10", |b| b.iter(||
    multi_thread_tester(black_box(10), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_channel))));
  c.bench_function("set get new oneshot channel futures 10", |b| b.iter(||
    multi_thread_tester(black_box(10), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_oneshot_channel_futures))));
  c.bench_function("set get new oneshot channel tokio 10", |b| b.iter(||
    multi_thread_tester(black_box(10), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_oneshot_channel_tokio))));

  c.bench_function("set get unsafe 100", |b| b.iter(||
    multi_thread_tester(black_box(100), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_unsafe))));
  c.bench_function("set get new channel 100", |b| b.iter(||
    multi_thread_tester(black_box(100), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_channel))));
  c.bench_function("set get new oneshot channel futures 100", |b| b.iter(||
    multi_thread_tester(black_box(100), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_oneshot_channel_futures))));
  c.bench_function("set get new oneshot channel tokio 100", |b| b.iter(||
    multi_thread_tester(black_box(100), black_box(10), black_box(&thingy), black_box(ThingyController::set_and_get_new_oneshot_channel_tokio))));

  thingy.controller_stop_thread();
  thingy_handle.join().unwrap();
}

criterion_group!(
  name=benches;
  config=Criterion::default().sample_size(1000);
  targets = channel_options_benchmark
);
criterion_main!(benches);
