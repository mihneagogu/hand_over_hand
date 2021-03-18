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

/// This is an implementation of the "find" method for concurrent sets implemented as sorted linked
/// lists. "find" is used by all "remove", "push", "contains" functions and returns two guards for
/// the nodes where the (new) value needs to be put or removed from. The general idea is that
/// we always hold at least a lock in our hands, but most of the time two locks for adjacent nodes,
/// so that no other thread can overtake us when calling find. This ensures that we never deadlock
/// because of the locking order, and also we know it is a sound way of doing the locking and it
/// enables parallelism for disjoint parts of the list. This "find" is just a sketch, but properly
/// shows how the hand-over-hand locking needs to be done. In particular, there are two things we
/// are missing. One, the LinkedList which consists of a head (Node) should be generic over T, but
/// we are using i32 for demonstrational purposes (also possibly covariant, but that's beyond the
/// scope of this). Secondly, the LinkedList should have backwards references as well for "remove"
/// operations. That is not done here, but can easily be integrated with Weak references
/// (std::sync::Weak), instead of Arcs for references to previous nodes.
/// More importantly, the locking algorithm (and linked lists in general for that matter) is very
/// rust-counterintuitive, since it requries that from one locked node we get another locked node
/// via the reference, which the compiler totally does not like. What we do here to get around that
/// is basically turn off the compiler checks by artifically extending the lifetimes of the
/// MutexGuards so that we can end up with two owned guards (using ManullyDrop along with
/// mem::transmute). Because of this, all memory freeing is done by hand. 
/// "find" shows that you can even design hand-over-hand
/// locking for linked lists in Rust, albeit with a lot of unsafe. Generally, proper implementations
/// which are rust-idiomatic and fast would use Unique or NonNull and atomic indices, but this was
/// done just to prove that you can even model concurrent linked lists in Rust.
/// SAFETY:
/// For the call to be safe, we need the caller to NEVER EVER do this:
/// let (node, node_next) = find(&list_head);
/// node.next = None;
///  node_next.next  = None; 
/// This is because the node.next would drop the previous Option<Arc<Mutex>>, but node is actually
/// a guard of that mutex, so it would trigger a use-after-free. The right way mutate node.next 
/// is to do this:
/// let (node, node_next) = find(&list_head);
/// let mutex = node.next.take();
///    *** mutation ***
/// ManuallyDrop::into_inner(node); <- drop the guard
/// drop(mutex);                    <- drop the actual mutex, if needed
/// Like this, we ensure that we do not use-after-free by dropping the guard when the mutex is
/// freed
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
    let mutex = f.next.take(); // Do not free the mutex, since we are holding its guard
    println!("{:?}", f);
    println!("s val {}", s.val);

    let _ = ManuallyDrop::into_inner(f); // Drop the guard
    let _ = ManuallyDrop::into_inner(s);
    drop(mutex); // Since we dropped the guard we are free to drop the mutex as well if we want to

}
