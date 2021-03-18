# hand_over_hand
This is a toy repo to demonstrate hand-over-hand locking for linked lists, but in Rust. While in almost any other language (original inspiration drawn from a Java implementation) this is trivial, as pretty much absolutely everyone knows, because of Rust's memory model and views on memory lifetimes and aliasing and referencing, linked lists are a huge pain in Rust and require unsafe. This is also because of the way of thinking of the "next" and "previous" pointers. I tried all of this just to prove myself that this behaviour is stil perfectly representable via Rust, which is true, but it does require quite some unsafe to tell the compiler "don't worry i gotttttttt it". A small view on the implementation of "find" and where it comes from:
We are simulating an implementation (which is not specified here) of concurrent sets modelled as sorted linked list. For all "push", "remove", "contains" we need a "find" function which returns two locked nodes (ManuallyDrop<MutexGuard<Node>>) which are adjacent which contain the position in the list where our data needs to be added (or removed from).

The tactic works like this: you start from the head of the list and lock nodes two at a time (unless there are 0 or 1 elements in the list. We omit this case for simplicity):

N1 -> N2 -> N3 -> N4 ...
Locking is done like this: lock N1, lock N2. Drop lockN1, acquire lock N3. Drop lock N2, acquire lock N4 etc... This means that at all times we have at least one lock in our hands, meaning no other thread can overtake us in mutating (or inspecting really) the list. The reason this is hard in rust is because we are trying to get the second lock from a reference tied to the lifetime of the first lock, which goes out of scope at the end of the loop (and gets dropped, in normal Rust).
Consider the pseudocode:

let pred, curr; // Both lockable nodes
pred = head;
pred.lock();
curr = head.next();
curr.lock();

while (curr.key < desiredKey) {
  pred.unlock();
  pred = curr;
  curr = curr.next;
  curr.lock();
}
return (pred, curr); // Two nodes with their mutexes locked

This looks all and well, but the only problem is curr's MutexGuard is obtained from pred, but pred is dropped at the end of the scope of the current iteration, so this invalidates both references. And that is totally right. However, we need both guards to be available in every iteration, so there is no way of doing that without extending their lifetimes. The way this is done is by using std::mem::ManuallyDrop (to deny the compiler the drop) and then by transmuting pred to actually extend its lifetime. This means we go from letting the compiler do the freeing to managing the freeing and releasing of MutexGuards on our own, and also completely disabling the lifetime checks of the compiler. In this case, we know this is the right thing to do, so long as we always remember to unlock the nodes we don't need anymore, by calling ManuallyDrop::into_inner() on their guards.
