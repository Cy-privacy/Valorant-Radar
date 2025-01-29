// Virtual to physical conversion
use std::sync::Mutex;
use std::time::Instant;

use crate::driver::ReadPhysicalMemory;

use super::DriverContext;
use once_cell::sync::Lazy;

const TLB_ENTRY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

// Thread-safe global TLB (dtb, virt_page, phys_page, timestamp)
// I know this isnt the best implementation but it worked well enough.
#[allow(clippy::type_complexity)]
static TLB: Lazy<Mutex<std::collections::HashMap<(u64, u64), (u64, Instant)>>> =
    Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

impl DriverContext {
    pub fn translate_linear_address(&self, dtb: u64 /* cr3 */, virt_addr: u64) -> u64 {
        let virtual_page = virt_addr & !0xfff;
        const PAGE_OFFSET_SIZE: u64 = 12;
        const PMASK: u64 = 0x000F_FFFF_F000;
        let dir_table = dtb & !0xf;

        let page_offset: u64 = virt_addr & !(!0u64 << PAGE_OFFSET_SIZE);

        if virt_addr == 0 {
            return 0;
        }

        // Look through the driver self TLB
        let mut tlb = TLB.lock().unwrap();
        tlb.retain(|_, translation| translation.1.elapsed() < TLB_ENTRY_TIMEOUT);
        if let Some(translation) = tlb.get(&(dtb, virtual_page)) {
            return translation.0 + page_offset;
        }

        let p_te = (virt_addr >> 12) & (0x1ffu64);
        let p_t = (virt_addr >> 21) & (0x1ffu64);
        let p_pd = (virt_addr >> 30) & (0x1ffu64);
        let p_dp = (virt_addr >> 39) & (0x1ffu64);

        let pdpe_addr: u64 = self.read_physical_memory(dir_table + 8 * p_dp);

        if pdpe_addr == 0 {
            return 0;
        }

        let p_de_addr: u64 = self.read_physical_memory((pdpe_addr & PMASK) + 0x8 * p_pd);

        if p_de_addr == 0 {
            return 0;
        } else if p_de_addr & 0x80 != 0 {
            return (p_de_addr & (!0u64 << 42 >> 12)) + (virt_addr & !(!0u64 << 30));
        }

        let p_te_addr: u64 = self.read_physical_memory((p_de_addr & PMASK) + (8 * p_t));

        if p_te_addr & 0x80 != 0 {
            return (p_te_addr & PMASK) + (virt_addr & !(!0u64 << 21));
        }

        let virt_addr: u64 = self.read_physical_memory((p_te_addr & PMASK) + p_te * 8);

        let pa_page = virt_addr & PMASK;

        // Add the translation to the TLB
        {
            let mut tlb = TLB.lock().unwrap();
            tlb.insert((dtb, virtual_page), (pa_page, Instant::now()));
        }
        pa_page + page_offset
    }
}

#[allow(unused_variables)]
impl ReadPhysicalMemory for DriverContext {
    fn read_physical_memory<T>(&self, addr: u64) -> T {
        // * Implement Physical Memory Reads here
        /* let page_offset: u64 = addr & 0xfff;
        let memory_page = addr & !0xfff;
        let size_to_read = std::mem::size_of::<T>() as u64; */

        unsafe { std::mem::zeroed() }
    }

    fn read_raw_physical_memory(&self, addr: u64, out_buffer: *mut u8, size: usize) -> bool {
        /*  let page_offset: u64 = addr & 0xfff;
        let memory_page = addr & !0xfff; */

        unsafe {
            std::ptr::write_bytes(out_buffer, 0, size);
        }
        false
    }
}
