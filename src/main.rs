use std::{thread};//, time};
use crossbeam_channel::{unbounded, Sender};//, Receiver};
use std::cell::RefCell;

// JPB: TODO: Make this private
struct Thingy {
  a: RefCell<i32>,
}

impl Thingy {
  pub fn new(i: i32) -> Thingy { Thingy{a: RefCell::new(i)} }
  pub fn print_a(&self) { println!("a = {}", self.a.borrow()); }
  //pub fn inc_a(&self, i: i32) { let x = self.a.borrow_mut(); x += i; }
  pub fn inc_a(&self, i: i32) { self.a.replace_with(|&mut old| old + i); }
  pub fn print_hello() { println!("Hello"); }
}

enum ThingyFuncs {
  // Internal commands
  ThingyWorkerQuit(),

  // Function duplicates
  PrintA(),
  IncA(i32),
  PrintHello(),
}

struct ThingyWorker {}
impl ThingyWorker {
  pub fn new(i: i32) -> (thread::JoinHandle<()>, ThingyController) {
    let (send, recv) = unbounded::<ThingyFuncs>();
    let thingy = Thingy::new(i);
    let handle = thread::spawn(move || {
      loop {
        match recv.recv().unwrap() {
          ThingyFuncs::ThingyWorkerQuit() => break,
          ThingyFuncs::PrintA() => thingy.print_a(),
          ThingyFuncs::IncA(i) => thingy.inc_a(i),
          ThingyFuncs::PrintHello() => Thingy::print_hello(),
        }
      }
    });
    (handle, ThingyController{send})
  }
}

struct ThingyController {
  send: Sender<ThingyFuncs>,
}

impl ThingyController {
  pub fn print_a(&self) {
    self.send.send(ThingyFuncs::PrintA()).unwrap();
  }
  pub fn inc_a(&self, i: i32) {
    self.send.send(ThingyFuncs::IncA(i)).unwrap();
  }
  pub fn print_hello(&self) {
    self.send.send(ThingyFuncs::PrintHello()).unwrap();
  }
}

fn main() {
  let (handle, thingy) = ThingyWorker::new(-1);

  //thread::sleep(time::Duration::from_millis(500));

  thingy.print_hello();
  thingy.print_a();
  thingy.inc_a(5);
  thingy.print_a();
  thingy.send.send(ThingyFuncs::ThingyWorkerQuit()).unwrap();

  handle.join().unwrap();
}
