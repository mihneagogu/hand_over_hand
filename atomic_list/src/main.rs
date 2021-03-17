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

    // SAFETY: There is no other reference using prev_cell. So we are safe to use it here
    // We use it via the UnsafeCell because while getting the next node we really don't want
    // to move. All of this is also sustained by the fact that prev is alive until the start of the
    // loop.
    let mut curr = unsafe {
        ManuallyDrop::new((&*prev_cell.get()).next.as_ref().unwrap().lock().unwrap())
    };

    let mut prev = prev_cell.into_inner();
    loop {
        // Now that we own two adjacent locks, release the first one so we can keep traversing
        // the list. 
        let _ = ManuallyDrop::into_inner(prev);
        prev = curr;
        let prev_cell: UnsafeCell<ManuallyDrop<MutexGuard<Node>>> = prev.into();

        // SAFETY: This is the only use of the reference. This is used so we do not 
        // move prev into this call
        curr = unsafe {
            ManuallyDrop::new((&*prev_cell.get()).next.as_ref().unwrap().lock().unwrap())
        };
        prev = prev_cell.into_inner();
        // SAFETY: This is the most important aspect of the hand over hand locking:
        // we are extending the liftime of prev. This call is safe because we know prev
        // lives until the start of the next iteration (unless we break out), because
        // it is a ManuallyDrop, so the compiler won't try to free it and pull the rug from under
        // us
        prev = unsafe { std::mem::transmute::<ManuallyDrop<MutexGuard<Node>>, ManuallyDrop<MutexGuard<Node>>>(prev) };
        if curr.val == 4 {
            // Desired value found, return the owned nodes. The caller must ensure
            // that ManuallyDrop::into_inner is called on the two guards so that they are released
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
