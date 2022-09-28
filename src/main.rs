use std::{thread, time};
use crossbeam_channel::{unbounded, Sender};//, Receiver};

#[derive(Debug)]
struct Func<T> {
  func: fn(T),
  arg: T,
}

//#[derive(Debug)]
struct Worker {
  send : Sender<Func<i32>>,
  //recv : Receiver<i32>,
}

impl Worker {
  pub fn new() -> Worker {
    let (send, recv) = unbounded::<Func<i32>>();
    thread::spawn(move || {
      //loop {
      for _ in 0..5 {
        let msg = recv.recv().unwrap();
        println!("Received {:?}", msg);
        (msg.func)(msg.arg);
      }
    });
    Worker{send}
  }

  pub fn print_hello(i: i32) { println!("hello {}", i); }
}

fn main() {
  let w1 = Worker::new();

  for i in 0..5 {
    let func = Func {func: Worker::print_hello, arg: i};
    w1.send.send(func).unwrap();
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
