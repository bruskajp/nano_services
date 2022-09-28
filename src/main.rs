use std::{thread, time};
use crossbeam_channel::{unbounded, Sender, Receiver};

//#[derive(Debug)]
struct Worker {
  send : Sender<i32>,
  //recv : Receiver<i32>,
}

impl Worker {
  pub fn new() -> Worker {
    let (send, recv) = unbounded();
    thread::spawn(move || {
      //loop {
      for _ in 0..5 {
        let msg = recv.recv().unwrap();
        println!("Received {}", msg);
      }
    });
    Worker{send}
  }

  //fn print_hello() { println!("hello"); }

  //struct Process() {}

  //struct PrintHello() {}

  //impl PrintHello {
  //  fn process() { print_hello(); }
  //}
}

fn main() {
  let w1 = Worker::new();
  //println!("{}", w1.send);
  //w1.subscribe();
  //a.a = 6;
  //w1.start();

  for i in 0..5 {
    w1.send.send(i).unwrap();
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
