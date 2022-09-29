use std::{thread, time};
use crossbeam_channel::{unbounded, Sender};//, Receiver};

struct Func {
  //func: fn(T),
  data: Box<dyn Fn(&Worker, fn(&Worker)) + Send>,
}

impl Func {
  fn invoke(&self, worker: &Worker, func: fn(&Worker)) {
        (self.data)(worker, func)
  }
}

struct WorkerController {
  send : Sender<Func>,
  //recv : Receiver<i32>,
}

impl WorkerController {
  pub fn print_hello(&self, i: i32) {
    let x = move |worker: &Worker, func: fn(&Worker)| {func(worker);};
    let func = Func {data: Box::new(x)};
    self.send.send(func).unwrap();
  }
}

struct Worker {
  //a : i32,
}

impl Worker {
  pub fn new() -> WorkerController {
    let (send, recv) = unbounded::<Func>();
    let worker = Worker{};
    thread::spawn(move || {
      //loop {
      for _ in 0..5 {
        let msg = recv.recv().unwrap();
        //println!("Received {:?}", msg);
        //(msg.func)(msg.arg);
        msg.invoke(&worker, Worker::print_hello_helper);
      }
    });
    WorkerController{send}
  }

  //fn print_hello_helper(&self, i: i32) { println!("hello {}", i); }
  fn print_hello_helper(&self) { println!("hello {}", 1); }
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
