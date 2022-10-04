
// ------------------------------------
// API NOTES
// 
// 1) All functions must use owned passing (no references) for thread safety (stop deadlocks)
// 2) The original class (Thingy) must have a constructor ("new" function)
// 3) The worker is created by calling <original_class_name>Worker::new()
// ------------------------------------

use std::cell::RefCell;

#[macro_use]
extern crate my_macro;

#[intro]
struct Thingy {
  a: RefCell<i32>,
}

#[worker]
impl Thingy {
  pub fn new(i: i32) -> Thingy { Thingy{a: RefCell::new(i)} }
  pub fn print_a(&self) { println!("a = {}", self.a.borrow()); }
  pub fn inc_a(&self, i: i32) { self.a.replace_with(|&mut old| old + i); }
  pub fn inc_a_twice(&self, i: i32, j: i32) { self.a.replace_with(|&mut old| old + i + j); }
  pub fn print_hello() { println!("Hello"); }
  fn internal(&self) {println!("internal");}
}

//enum WorkerFuncs {
//  // Internal commands
//  WorkerQuit(),
//
//  // Function duplicates
//  PrintA(),
//  IncA(i32),
//  IncATwice(i32, i32),
//}
//
//struct ThingyWorker;
//impl ThingyWorker {
//  pub fn new(i: i32) -> (thread::JoinHandle<()>, ThingyController) {
//    let (send, recv) = unbounded::<Box<WorkerFuncs>>();
//    let thingy = Thingy::new(i);
//    let handle = thread::spawn(move || {
//      loop {
//        match *recv.recv().unwrap() {
//          WorkerFuncs::WorkerQuit() => break,
//          WorkerFuncs::PrintA() => thingy.print_a(),
//          WorkerFuncs::IncA(i) => thingy.inc_a(i),
//          WorkerFuncs::IncATwice(i, j) => thingy.inc_a_twice(i, j),
//        }
//      }
//    });
//    (handle, ThingyController{send})
//  }
//}
//
//struct ThingyController {
//  send: Sender<Box<WorkerFuncs>>,
//}
//
//impl ThingyController {
//  // Internal commands
//  pub fn controller_stop_thread(&self) {
//    self.send.send(Box::new(WorkerFuncs::WorkerQuit())).unwrap();
//  }
//
//  // Function duplicates
//  pub fn print_a(&self) {
//    self.send.send(Box::new(WorkerFuncs::PrintA())).unwrap();
//  }
//  pub fn inc_a(&self, i: i32) {
//    self.send.send(Box::new(WorkerFuncs::IncA(i))).unwrap();
//  }
//  pub fn inc_a_twice(&self, i: i32, j: i32) {
//    self.send.send(Box::new(WorkerFuncs::IncATwice(i, j))).unwrap();
//  }
//}

fn main() {
  Thingy::introspect();

  let (handle, thingy) = ThingyWorker::new(-1);

  Thingy::print_hello();
  thingy.print_a();
  thingy.inc_a(5);
  thingy.print_a();
  thingy.inc_a_twice(1, 5);
  thingy.print_a();
  thingy.controller_stop_thread();

  handle.join().unwrap();
}
