use std::{thread, time};
use crossbeam_channel::{unbounded, Sender};//, Receiver};

enum Funcs {
  PrintHello(),
  PrintInt(i32),
}

struct WorkerController {
  send : Sender<Funcs>,
  //recv : Receiver<i32>,
}

impl WorkerController {
  pub fn print_hello(&self, i: i32) {
    self.send.send(Funcs::PrintInt(i)).unwrap();
  }
}

struct Worker {
  //a : i32,
}

impl Worker {
  pub fn new() -> WorkerController {
    let (send, recv) = unbounded::<Funcs>();
    let worker = Worker{};
    thread::spawn(move || {
      //loop {
      for _ in 0..5 {
        match recv.recv().unwrap() {
          Funcs::PrintHello() => worker.print_hello_original(42),
          Funcs::PrintInt(i) => worker.print_hello_original(i),
        }
      }
    });
    WorkerController{send}
  }

  //fn print_hello_original(&self, i: i32) { println!("hello {}", i); }
  fn print_hello_original(&self, i: i32) { println!("hello {}", i); }
}

fn main() {
  let w1 = Worker::new();

  for i in 0..5 {
    w1.print_hello(i);
    thread::sleep(time::Duration::from_millis(250));
  }

  //crossbeam::scope(|s| {
  //  s.spawn(|_| {
  //    for i in 0..5 {
  //      w1.send.send(i).unwrap();
  //      thread::sleep(time::Duration::from_millis(250));
  //    }
  //  });
  //}).unwrap();
}
