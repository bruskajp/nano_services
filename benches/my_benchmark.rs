//use mycrate::fibonacci;

mod setup_raw;

use std::thread;
use setup_raw::{ThingyWorker, ThingyController};
use std::sync::{Arc, Mutex};

#[inline]
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn set_and_get_new_channel(num_events: i32, thingy: ThingyController, thingy_clone: ThingyController) {
  let handle = thread::spawn(move || {
    for i in 1..num_events {
      thingy_clone.set_and_get_new_channel(i);
    }   
  }); 

  for i in 1..num_events {
    thingy.set_and_get_new_channel(i);
  }

  handle.join().unwrap();
}

fn set_and_get_unsafe(num_events: i32, thingy: ThingyController, thingy_clone: ThingyController) {
  let handle = thread::spawn(move || {
    for i in 1..num_events {
      thingy_clone.set_and_get_unsafe(i);
    }   
  }); 

  for i in 1..num_events {
    thingy.set_and_get_unsafe(i);
  }

  handle.join().unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
  let counter = Arc::new(Mutex::new(-1));
  let (thingy_handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  
  c.bench_function("set get unsafe 1", |b| b.iter(|| set_and_get_unsafe(black_box(1), black_box(thingy.clone()), black_box(thingy.clone()))));
  c.bench_function("set get new channel 1", |b| b.iter(|| set_and_get_new_channel(black_box(1), black_box(thingy.clone()), black_box(thingy.clone()))));
  c.bench_function("set get unsafe 10", |b| b.iter(|| set_and_get_unsafe(black_box(10), black_box(thingy.clone()), black_box(thingy.clone()))));
  c.bench_function("set get new channel 10", |b| b.iter(|| set_and_get_new_channel(black_box(10), black_box(thingy.clone()), black_box(thingy.clone()))));
  c.bench_function("set get unsafe 100", |b| b.iter(|| set_and_get_unsafe(black_box(100), black_box(thingy.clone()), black_box(thingy.clone()))));
  c.bench_function("set get new channel 100", |b| b.iter(|| set_and_get_new_channel(black_box(100), black_box(thingy.clone()), black_box(thingy.clone()))));

  thingy.controller_stop_thread();
  thingy_handle.join().unwrap();

}

//pub fn criterion_benchmark(c: &mut Criterion) {
//  c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
//}

//pub fn criterion_benchmark(c: &mut Criterion) {
//  let counter = Arc::new(Mutex::new(-1));
//  let (thingy_handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
//  
//  let handle = thread::spawn(|| {
//    for i in 1..10 {
//      c.bench_function("set get new chanel 1", |b| b.iter(|| thingy.set_and_get_b(black_box(i)) ));
//    }   
//  }); 
//
//  for i in 1..10 {
//    c.bench_function("set get new chanel 2", |b| b.iter(|| thingy.set_and_get_b(black_box(i)) ));
//  }
//
//  handle.join().unwrap();
//
//  thingy.controller_stop_thread();
//  thingy_handle.join().unwrap();
//
//}


criterion_group!(
  name=benches;
  config=Criterion::default().sample_size(2000);
  targets = criterion_benchmark
);
criterion_main!(benches);
