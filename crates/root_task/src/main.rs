#![no_std]
#![no_main]
mod allocator;

use allocator::ObjectAllocator;
use sel4::debug_println;
use sel4_root_task::{Never, root_task};
const TEST_ADDRESS: usize = 0xA000000000;

#[root_task]
fn main(bootinfo: &sel4::BootInfoPtr) -> sel4::Result<Never> {
    sel4::debug_println!("Hello, Skourios!");

    let mut allocator = ObjectAllocator::new(bootinfo);
    let frame = allocator
        .allocate(sel4::ObjectBlueprint::Arch(
            sel4::ObjectBlueprintArch::SmallPage,
        ))
        .cast::<sel4::cap_type::Granule>();

    for level in 1..sel4::vspace_levels::NUM_LEVELS {
        let tto = sel4::TranslationTableObjectType::from_level(level).unwrap();
        let result = allocator
            .allocate(tto.blueprint())
            .cast::<sel4::cap_type::UnspecifiedIntermediateTranslationTable>()
            .generic_intermediate_translation_table_map(
                tto,
                sel4::init_thread::slot::VSPACE.cap(),
                TEST_ADDRESS,
                sel4::VmAttributes::DEFAULT,
            );
        match result {
            Ok(_) => sel4::debug_println!("Level {} mapped OK", level),
            Err(e) => {
                self::debug_println!("Failed allocate and assign TTO at levle {} - {}", level, e)
            }
        }
    }

    let result = frame.frame_map(
        sel4::init_thread::slot::VSPACE.cap(),
        TEST_ADDRESS,
        sel4::CapRights::read_only(),
        sel4::VmAttributes::default(),
    );

    match result {
        Ok(_) => sel4::debug_println!("Frame mapped OK"),
        Err(e) => {
            self::debug_println!("Failed to map frame {}", e)
        }
    }

    let read_addr = TEST_ADDRESS as *const u32;
    let write_addr = TEST_ADDRESS as *mut u32;
    unsafe {
        sel4::debug_println!("Contents of memory - {:#x}", *read_addr);
    }

    let result = frame.frame_map(
        sel4::init_thread::slot::VSPACE.cap(),
        TEST_ADDRESS,
        sel4::CapRights::read_write(),
        sel4::VmAttributes::default(),
    );

    match result {
        Ok(_) => sel4::debug_println!("Frame mapped OK"),
        Err(e) => {
            self::debug_println!("Failed to map frame {}", e)
        }
    }

    unsafe {
        *write_addr = 0xa5a57c7c;
        sel4::debug_println!("Updated contents of memory - {:#x}", *read_addr);
    }
    sel4::debug_println!("Stop me if you've heard this before!");
    sel4::init_thread::suspend_self()
}
