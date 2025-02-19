#![no_std]
#![no_main]

use sel4::{Cap, cap_type::Untyped, init_thread::Slot};
use sel4_root_task::{Never, root_task};
use sel4_sys::seL4_DebugCapIdentify;

fn find_idx(bootinfo: &sel4::BootInfoPtr, obj_size: usize) -> usize {
    bootinfo
        .untyped_list()
        .iter()
        .position(|slot| !slot.is_device() && slot.size_bits() >= obj_size)
        .unwrap()
}

fn retype(parent: &Cap<Untyped>, slot: &Slot, blueprint: sel4::ObjectBlueprint) -> () {
    let cnode = sel4::init_thread::slot::CNODE.cap();
    let result =
        parent.untyped_retype(&blueprint, &cnode.absolute_cptr_for_self(), slot.index(), 1);
    match result {
        Ok(()) => sel4::debug_println!("Allocated {:?}", blueprint),
        Err(e) => sel4::debug_println!("OOPSIE - failed to allocate {:?}, {}", blueprint, e),
    }
}

#[root_task]
fn main(bootinfo: &sel4::BootInfoPtr) -> sel4::Result<Never> {
    sel4::debug_println!("Hello, Skourios!");

    let total_size_bits = sel4::ObjectBlueprint::Tcb.physical_size_bits() + 1;

    let untyped_idx = find_idx(bootinfo, total_size_bits);
    let parent_untyped = bootinfo.untyped().index(untyped_idx).cap();

    let mut empty_slots = bootinfo
        .empty()
        .range()
        .map(sel4::init_thread::Slot::from_index);

    let child_slot = empty_slots.next().unwrap();
    let tcb_slot = empty_slots.next().unwrap();
    let endpoint = empty_slots.next().unwrap();
    let notification_slot = empty_slots.next().unwrap();

    retype(
        &parent_untyped,
        &child_slot,
        sel4::ObjectBlueprint::Untyped {
            size_bits: total_size_bits,
        },
    );

    let child_untyped = child_slot.downcast::<Untyped>().cap();
    retype(&child_untyped, &tcb_slot, sel4::ObjectBlueprint::Tcb);

    let tcb = tcb_slot.downcast::<sel4::cap_type::Tcb>().cap();

    retype(&child_untyped, &endpoint, sel4::ObjectBlueprint::Endpoint);
    retype(
        &child_untyped,
        &notification_slot,
        sel4::ObjectBlueprint::Notification,
    );
    let notification = notification_slot
        .downcast::<sel4::cap_type::Notification>()
        .cap();

    let result = tcb.tcb_bind_notification(notification);
    if !result.is_ok() {
        sel4::debug_println!("Failed to set notification");
    }

    let abs_child = sel4::init_thread::slot::CNODE
        .cap()
        .absolute_cptr(child_untyped);

    let result = abs_child.revoke();
    match result {
        Ok(()) => {
            sel4::debug_println!("revoked OK");
            sel4::debug_println!("TCB cap is now {}", seL4_DebugCapIdentify(tcb.bits()));
        }
        Err(e) => sel4::debug_println!("{}", e),
    }

    let num_copies = 1 << (total_size_bits - sel4::ObjectBlueprint::Endpoint.physical_size_bits());
    let result = child_untyped.untyped_retype(
        &sel4::ObjectBlueprint::Endpoint,
        &sel4::init_thread::slot::CNODE
            .cap()
            .absolute_cptr_for_self(),
        tcb_slot.index(),
        num_copies,
    );
    match result {
        Ok(()) => sel4::debug_println!("Created {} endpoints", num_copies),
        Err(e) => sel4::debug_println!("Failed to create copies - {}", e),
    }
    sel4::debug_println!("Stop me if you've heard this before!");
    sel4::init_thread::suspend_self()
}
