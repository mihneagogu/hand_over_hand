use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::cell::UnsafeCell;


#[derive(Debug)]
struct Node {
    val: i32,
    next: Option<Arc<Mutex<Node>>>,
}

fn find(head: &Arc<Mutex<Node>>) -> (ManuallyDrop<MutexGuard<Node>>, ManuallyDrop<MutexGuard<Node>>) {
   loop {
       // Artificially extend the lifetime of the guards so we can use them later
        let one = ManuallyDrop::new(head.lock().unwrap());
        let one_cell: UnsafeCell<ManuallyDrop<MutexGuard<Node>>> = one.into();
        let one = &one_cell;

        // SAFETY: one is alive as long as we want it to, so is two, so there is no 
        // way of deallocating either of them by mistake, so the references are valid
        // We need the UnsafeCell so that we can create "two" from "one"
        let two = unsafe {
            ManuallyDrop::new((&*one.get()).next.as_ref().unwrap().lock().unwrap())
        };
        let one = one_cell.into_inner();
        break (one, two)
    }
}

fn main() {
    println!("Hello, world!");
    let fourth = Node { val: 4, next: None };
    let third = Node { val: 3, next: Some(Arc::new(Mutex::new(fourth))) };
    let snd = Node { val: 2, next: Some(Arc::new(Mutex::new(third))) };
    let fst = Node { val : 1, next: Some(Arc::new(Mutex::new(snd))) };
    let fst = Arc::new(Mutex::new(fst));

    let (mut f, mut s) = find(&fst);

    println!("f val {}", f.val);
    f.next = None;
    println!("{:?}", f);
    println!("s val {}", s.val);

    let _ = ManuallyDrop::into_inner(f);
    let _ = ManuallyDrop::into_inner(s);
    // drop(fst);

}
