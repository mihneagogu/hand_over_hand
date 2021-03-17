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
    let prev = head;
    // Artificially extend the lifetime of the guards so we can use them later
    let prev = ManuallyDrop::new(prev.lock().unwrap());
    let prev_cell: UnsafeCell<ManuallyDrop<MutexGuard<Node>>> = prev.into();

    // SAFETY: There is no other reference using prev_cell.
    let mut curr = unsafe {
        ManuallyDrop::new((&*prev_cell.get()).next.as_ref().unwrap().lock().unwrap())
    };
    let mut prev = prev_cell.into_inner();
    loop {
        let _ = ManuallyDrop::into_inner(prev);
        prev = curr;
        let prev_cell: UnsafeCell<ManuallyDrop<MutexGuard<Node>>> = prev.into();

        curr = unsafe {
            ManuallyDrop::new((&*prev_cell.get()).next.as_ref().unwrap().lock().unwrap())
        };
        prev = prev_cell.into_inner();
        prev = unsafe { std::mem::transmute::<ManuallyDrop<MutexGuard<Node>>, ManuallyDrop<MutexGuard<Node>>>(prev) };
        if curr.val == 4 {
            break (prev, curr)
        }
    }
}

fn main() {
    println!("Hello, world!");
    let fourth = Node { val: 4, next: None };
    let third = Node { val: 3, next: Some(Arc::new(Mutex::new(fourth))) };
    let snd = Node { val: 2, next: Some(Arc::new(Mutex::new(third))) };
    let fst = Node { val : 1, next: Some(Arc::new(Mutex::new(snd))) };
    let fst = Arc::new(Mutex::new(fst));

    let (mut f, s) = find(&fst);

    println!("f val {}", f.val);
    f.next = None;
    println!("{:?}", f);
    println!("s val {}", s.val);

    let _ = ManuallyDrop::into_inner(f);
    let _ = ManuallyDrop::into_inner(s);
    // drop(fst);

}
