use std::{thread};//, time};
use crossbeam_channel::{unbounded, Sender};//, Receiver};
use std::cell::RefCell;

#[macro_use]
extern crate my_macro;

// Api Notes:
// 1) All functions must use owned passing (no references) for thread safety (stop deadlocks)



// JPB: TODO: Make this private
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
//  PrintHello(),
//}

//struct ThingyWorker;
impl ThingyWorker {
  pub fn new(i: i32) -> (thread::JoinHandle<()>, ThingyController) {
    let (send, recv) = unbounded::<Box<WorkerFuncs>>();
    let thingy = Thingy::new(i);
    let handle = thread::spawn(move || {
      loop {
        match *recv.recv().unwrap() {
          WorkerFuncs::WorkerQuit() => break,
          WorkerFuncs::PrintA() => thingy.print_a(),
          WorkerFuncs::IncA(i) => thingy.inc_a(i),
          WorkerFuncs::IncATwice(i, j) => thingy.inc_a_twice(i, j),
          //WorkerFuncs::PrintHello() => Thingy::print_hello(),
        }
      }
    });
    (handle, ThingyController{send})
  }
}

//struct ThingyController {
//  send: Sender<Box<WorkerFuncs>>,
//}

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
//  pub fn print_hello(&self) {
//    self.send.send(Box::new(WorkerFuncs::PrintHello())).unwrap();
//  }
//}

fn main() {
  Thingy::introspect();

  let (handle, thingy) = ThingyWorker::new(-1);

  //thread::sleep(time::Duration::from_millis(500));

  Thingy::print_hello();
  thingy.print_a();
  thingy.inc_a(5);
  thingy.print_a();
  thingy.inc_a_twice(1, 5);
  thingy.print_a();
  thingy.controller_stop_thread();

  handle.join().unwrap();
}
