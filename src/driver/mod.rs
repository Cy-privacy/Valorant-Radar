use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64};
use winapi::{self, um::winnt::HANDLE};

pub(crate) mod physical_memory;
pub(crate) mod read_virtual_memory;

#[allow(dead_code)]
pub struct DriverContext {
    pub driver_handle: HANDLE,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct KernelContext {
    pub base_virtual: u64,
    pub base_physical: u64,
    pub cr3_virt: u64,
    pub cr3_phys: u64,
    pub nt_header: IMAGE_NT_HEADERS64,
    pub dos_header: IMAGE_DOS_HEADER,
    pub image_section_header_ptr: u64,
    pub PsInitialSystemProcess_va: u64,
}
#[derive(Clone, Copy)]
pub struct ProcessContext {
    pub eprocess: u64,
    pub directory_table_base: u64, // cr3
    pub process_id: u32,
    pub base_addr: u64,
    pub nt_header: IMAGE_NT_HEADERS64,
    pub dos_header: IMAGE_DOS_HEADER,
    pub image_section_header_ptr: u64,
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self {
            eprocess: 0,
            directory_table_base: 0,
            process_id: 0,
            base_addr: 0,
            nt_header: unsafe { std::mem::zeroed() },
            dos_header: unsafe { std::mem::zeroed() },
            image_section_header_ptr: 0,
        }
    }
}
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct EProcessOffsets {
    pub system_base_offset: u64,
    pub directory_table_base: u64,
    pub image_file_name_offset: u64,
    pub unique_process_id_offset: u64,
    pub section_base_offset: u64,
    pub active_process_link_offset: u64,
    pub peb_offset: u64,

    pub object_table_offset: u64, //struct _HANDLE_TABLE* ObjectTable;
}

#[allow(dead_code)]
pub struct MemoryContext {
    pub driver_ctx: DriverContext,
    pub kernel_ctx: KernelContext,
    pub process_ctx: ProcessContext,
    pub shadow_memory_physical_address: u64, // Valorant specific
    pub eproc_ctx: EProcessOffsets,
}

impl Default for EProcessOffsets {
    fn default() -> Self {
        Self {
            system_base_offset: 0x0,
            directory_table_base: 0x28,
            image_file_name_offset: 0x5A8,
            unique_process_id_offset: 0x440,
            section_base_offset: 0x520,
            active_process_link_offset: 0x448,
            peb_offset: 0x550,
            object_table_offset: 0x570,
            // 1909 offsets build 18363
            /*  image_file_name_offset: 0x450,
            unique_process_id_offset: 0x2e8,
            section_base_offset: 0x3c8,
            active_process_link_offset: 0x2f0,
            peb_offset: 0x3f8,
            object_table_offset: 0x418,*/
        }
    }
}

pub fn init() -> Option<DriverContext> {
    // Initialize for physical memory reads here
    Some(DriverContext {
        driver_handle: 0 as HANDLE,
    })
}

pub trait ReadPhysicalMemory {
    fn read_physical_memory<T>(&self, addr: u64) -> T;
    fn read_raw_physical_memory(&self, addr: u64, out_buffer: *mut u8, size: usize) -> bool;
}
pub trait ReadVirtualMemory {
    fn read_virtual_memory<T>(&self, process_ctx: &ProcessContext, addr: u64) -> T;
    fn read_virtual_memory_into(
        &self,
        process_ctx: &ProcessContext,
        addr: u64,
        out_buffer: *mut u8,
        size: usize,
    ) -> bool;
}
