use crossbeam_epoch::{self as epoch, Atomic, Owned};
use std::sync::atomic::Ordering;

struct Node {
    data: u64,
}

fn main() {
    let node = Owned::new(Node { data: 42 } );
    let atomic_ptr = Atomic::from(node);

    let guard = &epoch::pin();

    let shared_ptr = atomic_ptr.load(Ordering::Acquire, guard);

    println!("Initial tag: {}", shared_ptr.tag());
    println!("Initial address: {:#018x}        --> last byte: {:08b}", shared_ptr.as_raw() as usize, shared_ptr.as_raw() as usize & 0xFF);
    println!("----------------------------------");

    let tagged_ptr_2 = shared_ptr.with_tag(2);
    let raw_bits_2 = unsafe { std::mem::transmute::<_, usize>(tagged_ptr_2) };

    println!("Tag: {}", tagged_ptr_2.tag());
    println!("Tagged address (tag 2): {:#018x} --> last byte: {:08b}", &raw_bits_2, &raw_bits_2 & 0xFF);
    println!("tagged_ptr_2.as_raw(): {:#018x}", tagged_ptr_2.as_raw() as usize);
    println!("----------------------------------");

    let tagged_ptr_3 = tagged_ptr_2.with_tag(3);
    let raw_bits_3 = unsafe { std::mem::transmute::<_, usize>(tagged_ptr_3) };

    println!("Tag: {}", tagged_ptr_3.tag());
    println!("Tagged address (tag 3): {:#018x} --> last byte: {:08b}", &raw_bits_3, &raw_bits_3 & 0xFF);
    println!("tagged_ptr_3.as_raw(): {:#018x}", tagged_ptr_3.as_raw() as usize);
    println!("----------------------------------");

    let node_ref: &Node = unsafe { tagged_ptr_3.deref() };
    let physical_addr = (node_ref as *const Node) as usize;

    println!("Clean physical address: {:#018x}", physical_addr);
    println!("Node inner data: {}", node_ref.data);
}
