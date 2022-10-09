
// ------------------------------------
// API NOTES
// 
// 1) All functions must use owned passing (no references) for thread safety (stop deadlocks)
// 2) The original class (Thingy) must have a constructor ("new" function)
// 3) The worker is created by calling <original_class_name>Worker::new()
// 4) As it stands, you cannot call Controller methods from other threads
//      - it creates a ordering error with the recvs in blocking methods
// ------------------------------------

use std::sync::{Arc, Mutex};

#[macro_use]
extern crate my_macro;

#[intro]
struct Thingy {
  a: Arc<Mutex<i32>>,
}

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
  pub fn set_and_get_a(&self, i: i32) -> i32 {
    *self.a.lock().unwrap() = i;
    self.a.lock().unwrap().clone()
  }
}

use std::thread;
use crossbeam_channel::{unbounded, Sender, Receiver};
enum WorkerFuncs {
    WorkerQuit(),
    AssertFalse(),
    PlusOneA(),
    IncA(i32),
    GetA(),
    SetAndGetA(i32),
}
enum WorkerReturns {
    GetA(i32),
    SetAndGetA(i32),
}
struct ThingyWorker;
impl ThingyWorker {
    pub fn new(a: Arc<Mutex<i32>>) -> (thread::JoinHandle<()>, ThingyController) {
        let (send_func, recv_func) = unbounded::<Box<WorkerFuncs>>();
        let (send_ret, recv_ret) = unbounded::<Box<WorkerReturns>>();
        let thingy = Thingy::new(a);
        let handle = thread::spawn(move || {
            loop {
                match *recv_func.recv().unwrap() {
                    WorkerFuncs::WorkerQuit() => break,
                    WorkerFuncs::AssertFalse() => thingy.assert_false(),
                    WorkerFuncs::PlusOneA() => thingy.plus_one_a(),
                    WorkerFuncs::IncA(i) => thingy.inc_a(i),
                    WorkerFuncs::GetA() => {
                        send_ret
                            .send(Box::new(WorkerReturns::GetA(thingy.get_a())))
                            .unwrap()
                    }
                    WorkerFuncs::SetAndGetA(i) => {
                        send_ret
                            .send(
                                Box::new(WorkerReturns::SetAndGetA(thingy.set_and_get_a(i))),
                            )
                            .unwrap()
                    }
                }
            }
        });
        (
            handle,
            ThingyController {
                send: send_func,
                recv: recv_ret,
            },
        )
    }
}

#[derive(Clone, Debug)]
struct ThingyController {
    send: Sender<Box<WorkerFuncs>>,
    recv: Receiver<Box<WorkerReturns>>,
}

impl ThingyController {
    pub fn controller_stop_thread(&self) {
        self.send.send(Box::new(WorkerFuncs::WorkerQuit())).unwrap();
    }
    pub fn assert_false(&self) {
        self.send.send(Box::new(WorkerFuncs::AssertFalse())).unwrap();
    }
    pub fn plus_one_a(&self) {
        self.send.send(Box::new(WorkerFuncs::PlusOneA())).unwrap();
    }
    pub fn inc_a(&self, i: i32) {
        self.send.send(Box::new(WorkerFuncs::IncA(i))).unwrap();
    }
    pub fn get_a(&self) -> i32 {
        self.send.send(Box::new(WorkerFuncs::GetA())).unwrap();
        match *self.recv.recv().unwrap() {
            WorkerReturns::GetA(ret) => ret,
            _ => panic!("Invalid return type in inc_and_get_a\n(may be using Controller class across threads)"),
        }
    }
    pub fn set_and_get_a(&self, i: i32) -> i32 {
        self.send.send(Box::new(WorkerFuncs::SetAndGetA(i))).unwrap();
        match *self.recv.recv().unwrap() {
            WorkerReturns::SetAndGetA(ret) => ret,
            _ => panic!("Invalid return type in inc_and_get_a\n(may be using Controller class across threads)"),
        }
    }
}

fn main() {
  let counter = Arc::new(Mutex::new(-1));
  let (thingy_handle, thingy) = ThingyWorker::new(Arc::clone(&counter));

  let thingy_clone = thingy.clone();
  let handle = thread::spawn(move || {
    assert_eq!(thingy_clone.set_and_get_a(3), 3);
  });

  assert_eq!(thingy.set_and_get_a(6), 6);

  thingy.controller_stop_thread();

  handle.join().unwrap();
  thingy_handle.join().unwrap();
}

