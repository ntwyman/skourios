use core::ops::Range;
use sel4;

pub(crate) struct ObjectAllocator {
    empty_slots: Range<usize>,
    untyped: sel4::cap::Untyped,
}

fn largest_untyped(bootinfo: &sel4::BootInfoPtr) -> sel4::cap::Untyped {
    let (index, _desc) = bootinfo
        .untyped_list()
        .iter()
        .enumerate()
        .filter(|(_idx, desc)| !desc.is_device())
        .max_by_key(|(_idx, desc)| desc.size_bits())
        .unwrap();
    return bootinfo.untyped().index(index).cap();
}

impl ObjectAllocator {
    pub(crate) fn new(bootinfo: &sel4::BootInfoPtr) -> Self {
        Self {
            empty_slots: bootinfo.empty().range(),
            untyped: largest_untyped(bootinfo),
        }
    }

    pub(crate) fn allocate(&mut self, blueprint: sel4::ObjectBlueprint) -> sel4::cap::Unspecified {
        let slot_index = self.empty_slots.next().unwrap();

        let r = self.untyped.untyped_retype(
            &blueprint,
            &sel4::init_thread::slot::CNODE
                .cap()
                .absolute_cptr_for_self(),
            slot_index,
            1,
        );

        match r {
            Ok(()) => sel4::debug_println!("Successfully allocated {:?}", blueprint.ty()),
            Err(e) => sel4::debug_println!("Failed to allocate {:?} - {}", blueprint.ty(), e),
        }
        sel4::init_thread::Slot::from_index(slot_index).cap()
    }

    // pub(crate) fn allocate_fixed_sized<T: sel4::CapTypeForObjectOfFixedSize>(
    //     &mut self,
    // ) -> sel4::Cap<T> {
    //     self.allocate(T::object_blueprint()).cast()
    // }
}
