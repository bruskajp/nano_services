
// ------------------------------------
// API NOTES
// 
// 1) All functions must use owned passing (no references) for thread safety (stop deadlocks)
// 2) The original class (Thingy) must have a constructor ("new" function)
// 3) The worker is created by calling <original_class_name>Worker::new()
// 4) As it stands, you cannot call Controller methods from other threads
//      - it creates a ordering error with the recvs in blocking methods
// ------------------------------------

use std::cell::RefCell;

#[macro_use]
extern crate my_macro;

#[intro]
struct Thingy {
  a: RefCell<i32>,
}

//#[worker]
impl Thingy {
  pub fn new(i: i32) -> Thingy { Thingy{a: RefCell::new(i)} }
  pub fn print_a(&self) { self.internal_print(&format!("a = {}", self.a.borrow())); }
  pub fn inc_a(&self, i: i32) { self.a.replace_with(|&mut old| old + i); }
  pub fn inc_a_twice(&self, i: i32, j: i32) { self.a.replace_with(|&mut old| old + i + j); }
  pub fn print_hello() { println!("Hello"); }
  fn internal_print(&self, val: &str) { println!("{val}"); }

  pub fn get_a(&self) -> i32 { self.a.borrow().clone() }
  pub fn inc_and_get_a(&self, i: i32) -> i32 { self.a.replace_with(|&mut old| old + i); self.a.borrow().clone() }
}

use std::{thread};
use crossbeam_channel::{unbounded, Sender, Receiver};

enum WorkerFuncs {
  // Internal commands
  WorkerQuit(),

  // Function duplicates
  PrintA(),
  IncA(i32),
  IncATwice(i32, i32),
  GetA(),
  IncAndGetA(i32),
}

enum WorkerReturns {
  GetA(i32),
  IncAndGetA(i32),
}

struct ThingyWorker;
impl ThingyWorker {
  pub fn new(i: i32) -> (thread::JoinHandle<()>, ThingyController) {
    let (send_func, recv_func) = unbounded::<Box<WorkerFuncs>>();
    let (send_ret, recv_ret) = unbounded::<Box<WorkerReturns>>();
    let thingy = Thingy::new(i);
    let handle = thread::spawn(move || {
      loop {
        match *recv_func.recv().unwrap() {
          WorkerFuncs::WorkerQuit() => break,
          WorkerFuncs::PrintA() => thingy.print_a(),
          WorkerFuncs::IncA(i) => thingy.inc_a(i),
          WorkerFuncs::IncATwice(i, j) => thingy.inc_a_twice(i, j),
          WorkerFuncs::GetA() => send_ret.send(Box::new(WorkerReturns::GetA(thingy.get_a()))).unwrap(),
          WorkerFuncs::IncAndGetA(i) => send_ret.send(Box::new(WorkerReturns::IncAndGetA(thingy.inc_and_get_a(i)))).unwrap(),
        }
      }
    });
    (handle, ThingyController{send: send_func, recv: recv_ret})
  }
}

struct ThingyController {
  send: Sender<Box<WorkerFuncs>>,
  recv: Receiver<Box<WorkerReturns>>,
}

impl ThingyController {
  // Internal commands
  pub fn controller_stop_thread(&self) {
    self.send.send(Box::new(WorkerFuncs::WorkerQuit())).unwrap();
  }

  // Function duplicates
  pub fn print_a(&self) -> () {
    self.send.send(Box::new(WorkerFuncs::PrintA())).unwrap();
  }
  pub fn inc_a(&self, i: i32) -> () {
    self.send.send(Box::new(WorkerFuncs::IncA(i))).unwrap();
  }
  pub fn inc_a_twice(&self, i: i32, j: i32) {
    self.send.send(Box::new(WorkerFuncs::IncATwice(i, j))).unwrap();
  }
  pub fn get_a(&self) -> i32 {
    self.send.send(Box::new(WorkerFuncs::GetA())).unwrap();
    match *self.recv.recv().unwrap() {
      WorkerReturns::GetA(ret) => ret,
      _ => { panic!("Invalid return type in get_a\n(may be using Controller class across threads)") },
    }
  }
  pub fn inc_and_get_a(&self, i: i32) -> i32 {
    self.send.send(Box::new(WorkerFuncs::IncAndGetA(i))).unwrap();
    match *self.recv.recv().unwrap() {
      WorkerReturns::IncAndGetA(ret) => ret,
      _ => panic!("Invalid return type in inc_and_get_a\n(may be using Controller class across threads)"),
    }
  }
}

fn main() {
  Thingy::introspect();

  let (handle, thingy) = ThingyWorker::new(-1);

  Thingy::print_hello();
  thingy.print_a();
  thingy.inc_a(5);
  thingy.print_a();
  thingy.inc_a_twice(1, 5);
  thingy.print_a();

  println!("get_a: {}", thingy.get_a() );
  println!("inc_and_get_a: {}", thingy.inc_and_get_a(5) );

  thingy.controller_stop_thread();

  handle.join().unwrap();
}
