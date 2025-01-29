/* use std::io::Write; */

use winapi::um::winnt::{
    IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_EXPORT_DIRECTORY, IMAGE_FILE_HEADER,
    IMAGE_NT_HEADERS64, IMAGE_SECTION_HEADER, LIST_ENTRY64,
};

use crate::driver::{
    DriverContext, KernelContext, MemoryContext, ProcessContext, ReadPhysicalMemory,
    ReadVirtualMemory,
};
use bitfield::bitfield;
bitfield! {
    pub struct PoolTrackerBigPages( [u8]);
    u64;
    pub va, set_va: 63, 0;
    pub key, set_key: 95, 64;
    pub pattern, set_pattern: 103, 96;
    pub pool_type, set_pool_type: 115, 104;
    pub slush_size, set_slush_size: 127, 116;
    pub number_of_bytes, set_number_of_bytes: 191, 128;
}

pub fn find_halp_low_stub(driver_ctx: &DriverContext) -> Option<(u64, u64)> {
    // ? PROCESSOR_START_BLOCK
    for i in (0..0x100000).step_by(0x1000) {
        let halp_low_stub: u64 = driver_ctx.read_physical_memory(i);
        if 0x00000001000600E9 != (0xffffffffffff00ff & halp_low_stub) {
            continue;
        }
        let kernel_cr3_virt_addr: u64 = driver_ctx.read_physical_memory(i + 0x070);
        if 0xfffff80000000000 != (0xfffff80000000003 & kernel_cr3_virt_addr) {
            continue;
        }
        let kernel_cr3_phys_addr: u64 = driver_ctx.read_physical_memory(i + 0x0A0);
        if 0xffffff0000000fff & kernel_cr3_phys_addr != 0 {
            continue;
        }
        if kernel_cr3_virt_addr != 0 && kernel_cr3_phys_addr != 0 {
            return Some((kernel_cr3_virt_addr, kernel_cr3_phys_addr));
        }
    }
    Some((0, 0))
}

pub fn get_kernel_base(driver_ctx: &DriverContext, cr3_pa: u64, cr3_va: u64) -> KernelContext {
    const PAGELK: u64 = 0x4B4C45474150;
    /*    let mut entry = cr3_pa;
    let signature: [u8; 2] = [0x4d, 0x5a]; */
    let start_address = !0xfffffffu64 & cr3_va;
    log::info!("Start Address: 0x{:X}", start_address);
    for va in (start_address..start_address + 0xfffffffff).step_by(0x100000) {
        let base = driver_ctx.translate_linear_address(cr3_pa, va);
        let image_dos_header: IMAGE_DOS_HEADER = driver_ctx.read_physical_memory(base);
        if image_dos_header.e_magic != IMAGE_DOS_SIGNATURE {
            continue;
        }
        if image_dos_header.e_lfanew < 0 || image_dos_header.e_lfanew > 800 {
            continue;
        }

        let image_nt_headers: IMAGE_NT_HEADERS64 =
            driver_ctx.read_physical_memory(base + image_dos_header.e_lfanew as u64);

        if image_nt_headers.Signature != 0x4550 {
            continue;
        }
        log::info!("Possible Base: 0x{:x}", base);
        if base & 0xfffff == 0 {
            let base_physical = base;
            // Last check
            let mut is_page_pool = false;
            for i in (0..0x1000).step_by(0x8) {
                let page_pool: u64 = driver_ctx.read_physical_memory(base_physical + i);
                if page_pool == PAGELK {
                    is_page_pool = true;
                    break;
                }
            }
            if !is_page_pool {
                continue;
            }
            let virtual_base = image_nt_headers.OptionalHeader.ImageBase;
            let optional_header_ptr = virtual_base
                + image_dos_header.e_lfanew as u64
                + std::mem::size_of::<IMAGE_FILE_HEADER>() as u64
                + 4; // Size of signature

            let image_section_header_ptr =
                optional_header_ptr + image_nt_headers.FileHeader.SizeOfOptionalHeader as u64;

            let mut kern_ctx = KernelContext {
                cr3_phys: cr3_pa,
                cr3_virt: cr3_va,
                base_physical: base,
                base_virtual: virtual_base,
                nt_header: image_nt_headers,
                dos_header: image_dos_header,
                PsInitialSystemProcess_va: 0,
                image_section_header_ptr,
            };
            let init_system_process =
                get_kernel_export(driver_ctx, &kern_ctx, "PsInitialSystemProcess");

            kern_ctx.PsInitialSystemProcess_va = init_system_process;
            if init_system_process != 0 {
                log::info!("ntOSKernel Found");
                log::info!("\t Base: Physical 0x{:X}", base);
                log::info!("\t Base: Virtual 0x{:X}", virtual_base);
                log::info!(
                    "\t Module Size 0x{:X}",
                    image_nt_headers.OptionalHeader.SizeOfImage
                );
                log::info!(
                    "\t Windows {}.{}",
                    image_nt_headers.OptionalHeader.MajorOperatingSystemVersion,
                    image_nt_headers.OptionalHeader.MinorOperatingSystemVersion
                );
                log::info!("PsInitialSystemProcess: 0x{:X}", init_system_process);
                return kern_ctx;
            }
        }
    }
    log::error!("Kernel Base not found");
    unsafe {
        KernelContext {
            cr3_phys: cr3_pa,
            cr3_virt: cr3_va,
            base_physical: 0,
            base_virtual: 0,
            nt_header: std::mem::zeroed(),
            dos_header: std::mem::zeroed(),
            PsInitialSystemProcess_va: 0,
            image_section_header_ptr: 0,
        }
    }
}

pub fn get_kernel_export(
    driver_ctx: &DriverContext,
    kern_ctx: &KernelContext,
    function_name: &str,
) -> u64 {
    // export address tables (EAT)
    let kernel_process = ProcessContext {
        eprocess: 0,
        directory_table_base: kern_ctx.cr3_phys,
        process_id: 4,
        base_addr: kern_ctx.base_virtual,
        nt_header: kern_ctx.nt_header,
        dos_header: kern_ctx.dos_header,
        image_section_header_ptr: kern_ctx.image_section_header_ptr,
    };
    let export_dir: IMAGE_EXPORT_DIRECTORY = driver_ctx.read_virtual_memory(
        &kernel_process,
        kernel_process.base_addr
            + kern_ctx.nt_header.OptionalHeader.DataDirectory[0].VirtualAddress as u64,
    );

    for i in 0..export_dir.NumberOfFunctions as u64 {
        let current_name_addr: u32 = driver_ctx.read_virtual_memory(
            &kernel_process,
            kernel_process.base_addr + u64::from(export_dir.AddressOfNames) + (i * 4),
        );
        let name_buf: [i8; 256] = driver_ctx.read_virtual_memory(
            &kernel_process,
            kernel_process.base_addr + u64::from(current_name_addr),
        );
        let name = unsafe {
            std::ffi::CStr::from_ptr(name_buf.as_ptr())
                .to_str()
                .unwrap()
                .trim()
        };

        if name == function_name {
            let ordinal: u16 = driver_ctx.read_virtual_memory(
                &kernel_process,
                kernel_process.base_addr + u64::from(export_dir.AddressOfNameOrdinals) + (i * 2),
            );
            let func_address: u32 = driver_ctx.read_virtual_memory(
                &kernel_process,
                kernel_process.base_addr
                    + u64::from(export_dir.AddressOfFunctions)
                    + u64::from(ordinal * 4),
            );
            return func_address as u64;
        }
    }
    0
}

#[allow(dead_code)]
pub fn search_signature_physical(
    ctx: &DriverContext,
    start_addr: u64,
    end_addr: u64,
    signature: &[u8],
    signature_mask: &[char],
) -> Option<u64> {
    let signature_length: usize = signature.len() - 1;
    let signature_mask_length: usize = signature_mask.len() - 1;
    if signature_mask_length != signature_length {
        log::error!("Mask length does not match signature length");
        return None;
    }

    for i in (start_addr..end_addr).step_by(0x1000) {
        let data: [u8; 0x1000] = ctx.read_physical_memory(i);
        for j in 0..0x1000 {
            if data[j] == signature[0] {
                let mut is_match = true;
                for k in 1..signature_length {
                    if data[j + k] != signature[k] && signature_mask[k] != '?' {
                        is_match = false;
                        break;
                    }
                }
                if is_match {
                    return Some(i + j as u64);
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
pub fn find_pattern_virtual_memory(
    ctx: &DriverContext,
    process: &ProcessContext,
    start_addr: u64,
    end_addr: u64,
    signature: &[u8],
    signature_mask: &[char],
) -> Option<u64> {
    let signature_length: usize = signature.len() - 1;
    let signature_mask_length: usize = signature_mask.len() - 1;
    if signature_mask_length != signature_length {
        log::error!("Mask length does not match signature length");
        return None;
    }

    for i in (start_addr..end_addr + 0x1000).step_by(0x1000) {
        let data: [u8; 0x1001] = ctx.read_virtual_memory(process, i);
        for j in 0..0x1000 - signature_length {
            if data[j] == signature[0] {
                //log::info!("Found first byte");
                let mut is_match = true;
                for k in 1..signature_length {
                    if data[j + k] != signature[k] && signature_mask[k] != '?' {
                        is_match = false;
                        break;
                    }
                }
                if is_match {
                    return Some(i + j as u64);
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
pub fn get_process(
    mem_ctx: &MemoryContext,
    process_name: &str,
    file_name: &str,
) -> Result<ProcessContext, ()> {
    let mut return_process_context = ProcessContext::default();

    let kernel_process = ProcessContext {
        eprocess: 0,
        directory_table_base: mem_ctx.kernel_ctx.cr3_phys,
        process_id: 4,
        base_addr: mem_ctx.kernel_ctx.base_virtual,
        ..Default::default()
    };

    let system_init_eprocess: u64 = mem_ctx.driver_ctx.read_virtual_memory(
        &kernel_process,
        kernel_process.base_addr + mem_ctx.kernel_ctx.PsInitialSystemProcess_va,
    );
    let mut current_eprocess: u64 = system_init_eprocess;

    loop {
        let process_id: u64 = mem_ctx.driver_ctx.read_virtual_memory(
            &kernel_process,
            current_eprocess + mem_ctx.eproc_ctx.unique_process_id_offset,
        );
        /*   log::info!(
            "Process ID  addr : {:X} trans {:X} value : {:X}",
            current_eprocess + mem_ctx.eproc_ctx.unique_process_id_offset,
            translate_linear_address(
                &mem_ctx.driver_ctx,
                kernel_process.directory_table_base,
                current_eprocess + mem_ctx.eproc_ctx.unique_process_id_offset
            ),
            process_id
        ); */
        let process_links: LIST_ENTRY64 = mem_ctx.driver_ctx.read_virtual_memory(
            &kernel_process,
            current_eprocess + mem_ctx.eproc_ctx.active_process_link_offset,
        ); // Size = 0x10;

        let process_name_buf: [u8; 16] = mem_ctx.driver_ctx.read_virtual_memory(
            &kernel_process,
            current_eprocess + mem_ctx.eproc_ctx.image_file_name_offset,
        );

        /* let current_process_name =
        String::from_utf8(process_name_buf.to_vec()).unwrap_or("".to_string()); */
        let current_process_name = unsafe {
            std::ffi::CStr::from_ptr(process_name_buf.as_ptr() as *const i8)
                .to_str()
                .ok()
                .unwrap_or("")
                .trim()
        };
        /*   log::info!(
            "Name : {}\tProcess ID : {:X}\tprocess_links.Flink {:X}\tCurEPROC :{:X}\tNext EPROC : {:X}",
            current_process_name,
            process_id,
            process_links.Flink,
            current_eprocess,
            process_links.Flink
                - mem_ctx.eproc_ctx.unique_process_id_offset
                - std::mem::size_of::<u64>() as u64
        ); */
        // Compare the process name to the one we're looking for

        if current_process_name == process_name || current_process_name == file_name
        //  || process_id != 18788
        {
            // Check if process is zombie process
            let object_table_pointer: u64 = mem_ctx.driver_ctx.read_virtual_memory(
                &kernel_process,
                current_eprocess + mem_ctx.eproc_ctx.object_table_offset,
            );
            //log::info!("object_table_pointer {:X}", object_table_pointer);
            if object_table_pointer != 0 {
                return_process_context.process_id = process_id as u32;
                return_process_context.eprocess = current_eprocess;
                return_process_context.directory_table_base =
                    mem_ctx.driver_ctx.read_virtual_memory(
                        &kernel_process,
                        current_eprocess + mem_ctx.eproc_ctx.directory_table_base,
                    );
                return_process_context.base_addr = mem_ctx.driver_ctx.read_virtual_memory(
                    &kernel_process,
                    current_eprocess + mem_ctx.eproc_ctx.section_base_offset,
                );

                // Get nt_headers and DOS headers
                let image_dos_header: IMAGE_DOS_HEADER = mem_ctx
                    .driver_ctx
                    .read_virtual_memory(&return_process_context, return_process_context.base_addr);

                let image_nt_headers: IMAGE_NT_HEADERS64 = mem_ctx.driver_ctx.read_virtual_memory(
                    &return_process_context,
                    return_process_context.base_addr + image_dos_header.e_lfanew as u64,
                );

                if image_nt_headers.Signature != 0x4550 {
                    log::error!("Invalid NT Signature");
                }
                return_process_context.dos_header = image_dos_header;
                return_process_context.nt_header = image_nt_headers;
                //
                /*     Signature: DWORD,
                FileHeader: IMAGE_FILE_HEADER, */
                //

                let optional_header_ptr = return_process_context.base_addr
                    + image_dos_header.e_lfanew as u64
                    + std::mem::size_of::<IMAGE_FILE_HEADER>() as u64
                    + 4; // Size of signature

                return_process_context.image_section_header_ptr =
                    optional_header_ptr + image_nt_headers.FileHeader.SizeOfOptionalHeader as u64;
                return Ok(return_process_context);
            } else {
                current_eprocess = process_links.Flink
                    - mem_ctx.eproc_ctx.unique_process_id_offset
                    - std::mem::size_of::<u64>() as u64;
                continue;
            }
        } else {
            current_eprocess = process_links.Flink
                - mem_ctx.eproc_ctx.unique_process_id_offset
                - std::mem::size_of::<u64>() as u64;
        }
        if current_eprocess == system_init_eprocess {
            return Err(());
        }
    }
}

pub fn get_big_pool_table_size(mem_ctx: &MemoryContext) -> Result<u64, ()> {
    let kernel_process = ProcessContext {
        eprocess: 0,
        directory_table_base: mem_ctx.kernel_ctx.cr3_phys,
        process_id: 4,
        base_addr: mem_ctx.kernel_ctx.base_virtual,
        nt_header: mem_ctx.kernel_ctx.nt_header,
        dos_header: mem_ctx.kernel_ctx.dos_header,
        image_section_header_ptr: mem_ctx.kernel_ctx.image_section_header_ptr,
    };

    let bigpool_table_size_sig = [0x4C, 0x8B, 0x15, 00, 00, 00, 00, 0x48, 0x85];
    let bigpool_table_size_mask: [char; 9] = "xxx????xx"
        .chars()
        .collect::<Vec<char>>()
        .try_into()
        .unwrap();

    let bigpool_table_size = find_pattern_virtual_memory(
        &mem_ctx.driver_ctx,
        &kernel_process,
        kernel_process.base_addr,
        kernel_process.base_addr
            + u64::from(mem_ctx.kernel_ctx.nt_header.OptionalHeader.SizeOfImage),
        &bigpool_table_size_sig,
        &bigpool_table_size_mask,
    );

    if bigpool_table_size.is_none() {
        log::error!("Failed to find bigpool_table_size");
        return Err(());
    }
    let bigpool_table_size = bigpool_table_size.unwrap();

    log::info!("bigpool_table_size instruction: {:X}", bigpool_table_size);

    let size_pointer = resolve_relative_address(mem_ctx, &kernel_process, bigpool_table_size, 3, 7);
    log::info!("BigPoolTableSize_pointer: {:X}", size_pointer);

    let u64_size: u64 = mem_ctx
        .driver_ctx
        .read_virtual_memory(&kernel_process, size_pointer);
    log::info!("BigPoolTableSize: {:X}", u64_size);
    Ok(u64_size)
}

pub fn get_big_pool_table(mem_ctx: &MemoryContext) -> Result<(u64, u64), ()> {
    let kernel_process = ProcessContext {
        eprocess: 0,
        directory_table_base: mem_ctx.kernel_ctx.cr3_phys,
        process_id: 4,
        base_addr: mem_ctx.kernel_ctx.base_virtual,
        nt_header: mem_ctx.kernel_ctx.nt_header,
        dos_header: mem_ctx.kernel_ctx.dos_header,
        image_section_header_ptr: mem_ctx.kernel_ctx.image_section_header_ptr,
    };

    let bigpool_instruction_sig: [u8; 15] = [
        0x48, 0x8B, 0x15, 00, 00, 00, 00, 0x4C, 0x8D, 0x0D, 00, 00, 00, 00, 0x4C,
    ];
    let bigpool_signature_mask: [char; 15] = "xxx????xxx????x"
        .chars()
        .collect::<Vec<char>>()
        .try_into()
        .unwrap();

    let bigpool_table_instruction = find_pattern_virtual_memory(
        &mem_ctx.driver_ctx,
        &kernel_process,
        kernel_process.base_addr,
        kernel_process.base_addr
            + u64::from(mem_ctx.kernel_ctx.nt_header.OptionalHeader.SizeOfImage),
        &bigpool_instruction_sig,
        &bigpool_signature_mask,
    );

    if bigpool_table_instruction.is_none() {
        log::error!("Failed to find bigpool_table");
        return Err(());
    }
    let bigpool_table_instruction = bigpool_table_instruction.unwrap();
    log::info!("BigPoolTableInstruction: {:X}", bigpool_table_instruction);

    let bigpool_table_ptr =
        resolve_relative_address(mem_ctx, &kernel_process, bigpool_table_instruction, 3, 7);

    log::info!("BigPoolTable: {:X}", bigpool_table_ptr);

    let bigpool_table = mem_ctx
        .driver_ctx
        .read_virtual_memory(&kernel_process, bigpool_table_ptr);
    /* let va: PoolTrackerBigPages<[u8; 0x18]> = mem_ctx
        .driver_ctx
        .read_virtual_memory(&kernel_process, bigpool_table);

    log::info!("PoolTrackerBigPages: {:X}", va.va());
    log::info!("PoolTrackerBigPages: {:X}", va.number_of_bytes()); */

    // We get the size of the big pool table now
    let bigpool_table_size = get_big_pool_table_size(mem_ctx);
    if bigpool_table_size.is_err() {
        log::error!("Failed to get big pool table size");
        return Err(());
    }

    Ok((bigpool_table, bigpool_table_size.ok().unwrap()))
}

#[allow(dead_code)]
pub fn dump_module(
    mem_ctx: &MemoryContext,
    process: &ProcessContext,
    module_base: u64,
    module_name: &str,
) {
    log::warn!("Dumper is not reliable! Use at your own risk!");
    use std::fs::File;
    use std::io::Write;

    let file_name = format!("{}.dump", module_name);
    let mut dump_file = match File::create(file_name) {
        Ok(file) => file,
        Err(err) => {
            log::error!("Failed to create file: {}", err);
            return;
        }
    };

    // Read the PE header to get the size of the PE file
    let dos_header = process.dos_header;
    let nt_header = process.nt_header;

    let pe_file_size = nt_header.OptionalHeader.SizeOfHeaders as usize;

    // Read the entire PE file into memory
    let mut pe_file_data = vec![0u8; pe_file_size];
    if !mem_ctx.driver_ctx.read_virtual_memory_into(
        process,
        module_base,
        pe_file_data.as_mut_ptr(),
        pe_file_size,
    ) {
        log::error!("Failed to read PE file data");
        return;
    }

    // Now we need to read the sections

    let section_count = nt_header.FileHeader.NumberOfSections as usize;
    let section_header_size = std::mem::size_of::<IMAGE_SECTION_HEADER>();
    let section_headers_offset =
        module_base + dos_header.e_lfanew as u64 + std::mem::size_of::<IMAGE_NT_HEADERS64>() as u64;

    let mut section_headers = Vec::with_capacity(section_count);
    for i in 0..section_count {
        let mut section_header: IMAGE_SECTION_HEADER = unsafe { std::mem::zeroed() }; // IMAGE_SECTION_HEADER used to implement default :(
        if !mem_ctx.driver_ctx.read_virtual_memory_into(
            process,
            section_headers_offset + (i * section_header_size) as u64,
            &mut section_header as *mut _ as *mut u8,
            section_header_size,
        ) {
            log::error!("Failed to read section header");
            return;
        }
        section_headers.push(section_header);
    }

    for i in 0..section_count {
        let section_header = &section_headers[i];
        let section_name = std::str::from_utf8(&section_header.Name).unwrap_or("ERROR");

        // Calculate the actual size of the section
        let section_size = if i < section_count - 1 {
            // For all sections except the last one, the size is the difference between the virtual addresses of this section and the next one
            section_headers[i + 1].VirtualAddress - section_header.VirtualAddress
        } else {
            // For the last section, the size is the SizeOfRawData field from the section header
            section_header.SizeOfRawData
        };

        log::info!(
            "Reading section {} (VirtualAddress: 0x{:X}, SizeOfRawData: 0x{:X})",
            section_name,
            section_header.VirtualAddress,
            section_size
        );

        let mut section_data = vec![0u8; section_size as usize];
        if !mem_ctx.driver_ctx.read_virtual_memory_into(
            process,
            module_base + section_header.PointerToRawData as u64,
            section_data.as_mut_ptr(),
            section_data.len(),
        ) {
            log::error!("Failed to read section data");
            return;
        }

        // Append the section data to the PE file data
        pe_file_data.extend_from_slice(&section_data);
    }

    // Write the PE file data to the dump file
    if let Err(err) = dump_file.write_all(&pe_file_data) {
        log::error!("Failed to write PE file data to dump file: {}", err);
    }
}

pub fn resolve_relative_address(
    mem_ctx: &MemoryContext,
    process: &ProcessContext,
    instruction: u64,
    offset_offset: u32,
    instruction_size: u32,
) -> u64 {
    let rip_offset = mem_ctx
        .driver_ctx
        .read_virtual_memory::<u32>(process, instruction + offset_offset as u64);
    instruction + instruction_size as u64 + rip_offset as u64
}

// Thank you Alex

macro_rules! global_singleton {
    ($name:ident, $type:ty) => {
        use std::sync::Arc;

        use once_cell::sync::Lazy;
        use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

        pub static $name: Lazy<Arc<RwLock<$type>>> =
            Lazy::new(|| Arc::new(RwLock::new(Default::default())));

        #[allow(dead_code)]
        impl $type {
            pub fn get() -> RwLockReadGuard<'static, $type> {
                $name.read()
            }

            pub fn get_mut() -> RwLockWriteGuard<'static, $type> {
                $name.write()
            }
        }
    };
}
pub(crate) use global_singleton;
