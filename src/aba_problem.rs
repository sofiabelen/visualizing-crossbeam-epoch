use std::time::Duration;
use crate::{AtomicPtr, Ordering};
use crate::thread;

pub struct Node<T> {
    value: T,
    next: *mut Node<T>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            next: std::ptr::null_mut(),
        }
    }
}

pub struct Stack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(std::ptr::null_mut())
        }
    }

    pub fn push(&self, value: T) {
        let mut current_head = self.head.load(Ordering::Relaxed);
        let node = Box::new(Node::new(value));
        let new_head = Box::into_raw(node);
        println!("[push] allocated new node at {:#x}",  new_head as usize);

        loop {
            unsafe { (*new_head).next  = current_head };

            match self.head.compare_exchange_weak(
                current_head,
                new_head, 
                Ordering::AcqRel, 
                Ordering::Relaxed
            ) {
                Ok(_) => {
                    println!("[push] CAS success: head {:#x} -> {:#x}",  current_head as usize, new_head as usize);
                    break;
                },
                Err(actual_head) => {
                    println!("[push] CAS failure: expected {}, actual {}", current_head as usize, actual_head as usize);
                    current_head = actual_head;
                },
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        self.pop_internal(false)
    }

    pub fn pop_with_delay(&self) -> Option<T> {
        self.pop_internal(true)
    }
    
    fn pop_internal(&self, inject_delay: bool) -> Option<T> {
        let mut current_head = self.head.load(Ordering::Relaxed);

        loop {
            if current_head.is_null() { return None; }

            // Safety: how do we know no other read is modifying this?
            let new_head = unsafe { (*current_head).next };
            println!("[pop] read head {:#x}, next: {:#x}", current_head as usize, new_head as usize);

            // Artificially added delay to trigger ABA
            if inject_delay {
                thread::sleep(Duration::from_millis(100));
            }

            //println!("trying CAS in push: current_head {:#x}, new_head {:#x}", current_head as usize, new_head as usize);
            match self.head.compare_exchange_weak(
                current_head,
                new_head, 
                Ordering::AcqRel, 
                Ordering::Relaxed) {
                
                Ok(_) => {
                    // Safety: as I'm writing this, rusts forces me to think about the safety of the unsafe operations,
                    // and the fact that I can't write a safety statement should be a red flag
                    println!("[pop] CAS success: head: {:#x} -> {:#x}", current_head as usize, new_head as usize);
                    let node = unsafe { Box::from_raw(current_head) };

                    return Some(node.value); // our ptr gets dropped as the Box goes out of scope
                },
                Err(actual_head) => {
                    println!("[pop] CAS fail: expected {:#x}, actual_head {:#x}", current_head as usize, actual_head as usize);
                    current_head = actual_head;
                }
            }
        }
    }
}

// SAFETY: Because raw pointers (*mut Node<T>) are NOT Send or Sync by default,
// you must explicitly tell the compiler that this type is safe to share across threads.
unsafe impl<T: Send> Send for Stack<T> {}
unsafe impl<T: Send> Sync for Stack<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arc;

    #[test]
    fn test_aba() {
        let stack = Arc::new(Stack::<i32>::new());
        let stack_clone = stack.clone();
        stack.push(2);
        stack.push(1);
        println!("---");
        println!();
        // stack at this point: head -> [1] -> [2] -> nullptr

        thread::scope(|s| {

            // This thread pops, but with a delay between reading head and the CAS loop
            // By the time it enters the CAS loop, the second thread has essentially
            // replaced the head, [1], with [3] that shares the same memory address as [1]
            // original: head -> [1] -> [2]
            // now     : head -> [3] -> nullptr
            s.spawn(|| {
                println!("thread 1 starts pop operation");
                let node = stack.pop_with_delay();

                // If this pop shows up as [3] this means the CAS succeeded,
                // and we were able to reproduce the bug, yay!
                println!("thread 1 continues pop: {:?}", node);
                println!("---");
                println!();
            });

            // During the first thread's delay window:
            // 1. This thread pops [1]
            // 2. Then pops [2]
            // 3. Pushes a new node [3], with the same address as [1]
            s.spawn(|| {
                // We wait a little to make sure the first thread gets a head (no pun intended) start
                thread::sleep(Duration::from_millis(20));

                println!();
                println!("---");
                println!("thread 2 pops: [{}]", stack_clone.pop().unwrap());
                println!("---");
                println!();
                println!("thread 2 pops: [{}]", stack_clone.pop().unwrap());
                println!("---");
                println!();
                let tmp_raw = Box::into_raw(Box::new(Node::new(43)));
                println!("tmp node created at {:#x}", tmp_raw as usize);

                // Let's hope the system heap allocator reuses the memory that was just freed
                // Stack now: head -> [3] -> nullptr
                println!("thread 2 pushes [3]");
                stack_clone.push(3);
                println!("---");
                println!();
            });

            thread::sleep(Duration::from_millis(500));
            // Current stack: head -> [3 (freed)]
            println!("Final pop: [{:?}]", stack.pop());
        });
    }
    
    #[test]
    fn test_basic_multithreading() {
        let stack = Arc::new(Stack::<i32>::new());
        let stack_clone = stack.clone();
        stack.push(1);
        stack.push(2);

        thread::scope(|s| {
            s.spawn(||{
                let val = stack.pop();
                assert!(val == Some(1) || val == Some(2));
            });

            s.spawn(||{
                let val = stack_clone.pop();
                assert!(val == Some(1) || val == Some(2));
            });
        });
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_basic() {
        let stack = Stack::<i32>::new();
        stack.push(10);
        stack.push(15);
        stack.push(13);
        stack.push(11);

        assert_eq!(stack.pop(), Some(11));
        assert_eq!(stack.pop(), Some(13));
        assert_eq!(stack.pop(), Some(15));
        assert_eq!(stack.pop(), Some(10));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_sequencial() {
        let stack = Stack::<i32>::new();
        stack.push(10);
        assert_eq!(stack.pop(), Some(10));
        stack.push(15);
        assert_eq!(stack.pop(), Some(15));
        stack.push(13);
        assert_eq!(stack.pop(), Some(13));
        stack.push(11);
        assert_eq!(stack.pop(), Some(11));
        assert_eq!(stack.pop(), None);
    }
}

// Notes:
// 1. realize that we don't need to pass self as &mut to pop and push
// 2. we need to put stack in an Arc, which will let us have shared references (remember we don't &mut, & is enough, that's the whole point)
// 3. we need to manually implemet the Send and Sync traits for Stack (do we need both?)
// 4. why didn't it crash with a segfault? 
//     In debug mode without AddressSanitizer (ASan), dereferencing recently freed memory that hasn't been returned to the OS kernel yet often silently reads junk data (like 0x0 / null) rather than triggering an instant hardware crash.