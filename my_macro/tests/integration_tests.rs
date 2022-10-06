use my_macro::*;

use std::sync::{Arc, Mutex};

struct Thingy {
  a: Arc<Mutex<i32>>,
}

#[worker]
impl Thingy {
  pub fn new(a: Arc<Mutex<i32>>) -> Thingy {
    *a.lock().unwrap() += 1;
    Thingy{a: a}
  }

  pub fn assert_false(&self) {
    assert!(false);
  }
  pub fn plus_one_a(&self) {
    Thingy::plus_one_internal(&self.a);
  }
  pub fn inc_a(&self, i: i32) {
    *self.a.lock().unwrap() += i;
  }

  fn plus_one_internal(a: &Arc<Mutex<i32>>) {
    *a.lock().unwrap() += 1;
  }

  pub fn get_a(&self) -> i32 {
    self.a.lock().unwrap().clone()
  }
  pub fn inc_and_get_a(&self, i: i32) -> i32 {
    *self.a.lock().unwrap() += i;
    self.a.lock().unwrap().clone()
  }
}

#[test]
fn create_and_close_worker() {
  let counter = Arc::new(Mutex::new(-1));
  let (handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  thingy.controller_stop_thread();
  handle.join().unwrap();
  
  assert_eq!(*counter.lock().unwrap(), 0);
}

#[test]
#[should_panic]
fn assert_false_in_worker() {
  let counter = Arc::new(Mutex::new(-1));
  let (handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  thingy.assert_false();
  thingy.controller_stop_thread();
  handle.join().unwrap();
}

#[test]
fn worker_method_no_args() {
  let counter = Arc::new(Mutex::new(-1));
  let (handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  thingy.plus_one_a();
  thingy.controller_stop_thread();
  handle.join().unwrap();
  
  assert_eq!(*counter.lock().unwrap(), 1);
}

#[test]
fn worker_method_with_args() {
  let counter = Arc::new(Mutex::new(-1));
  let (handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  thingy.inc_a(3);
  thingy.controller_stop_thread();
  handle.join().unwrap();
  
  assert_eq!(*counter.lock().unwrap(), 3);
}

#[test]
fn worker_blocking_method_no_args() {
  let counter = Arc::new(Mutex::new(-1));
  let (handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  assert_eq!(thingy.get_a(), 0);
  thingy.controller_stop_thread();
  handle.join().unwrap();
  
}

#[test]
fn worker_blocking_method_with_args() {
  let counter = Arc::new(Mutex::new(-1));
  let (handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  assert_eq!(thingy.inc_and_get_a(3), 3);
  thingy.controller_stop_thread();
  handle.join().unwrap();
  
}

#[test]
fn worker_check_ordering() {
  let counter = Arc::new(Mutex::new(-1));
  let (handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  thingy.inc_a(3);
  assert_eq!(thingy.get_a(), 3);
  thingy.controller_stop_thread();
  handle.join().unwrap();
  
}

#[test]
fn worker_check_ordering2() {
  let counter = Arc::new(Mutex::new(-1));
  let (handle, thingy) = ThingyWorker::new(Arc::clone(&counter));
  assert_eq!(thingy.inc_and_get_a(3), 3);
  assert_eq!(thingy.inc_and_get_a(3), 6);
  assert_eq!(thingy.inc_and_get_a(3), 9);
  thingy.controller_stop_thread();
  handle.join().unwrap();
  
}
