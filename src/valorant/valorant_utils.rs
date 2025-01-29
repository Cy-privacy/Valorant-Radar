use std::{collections::HashMap, sync::Mutex};

use once_cell::sync::Lazy;

use crate::{
    driver::{MemoryContext, ProcessContext, ReadPhysicalMemory, ReadVirtualMemory},
    utils::PoolTrackerBigPages,
    valorant::offsets,
};

use super::{structs::FNameEntry, ValorantContext};

pub trait ReadGame {
    fn read<T>(&self, address: u64) -> T;
    fn read_into(&self, address: u64, out_buffer: *mut u8, size: usize);
}

static FNAME_POOL_MAP: Lazy<Mutex<HashMap<u32, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl ValorantContext<'_, '_> {
    pub fn get_fname(&self, entry_id: u32) -> Result<String, String> {
        if let Some(value) = FNAME_POOL_MAP.lock().unwrap().get(&entry_id) {
            return Ok(value.clone());
        }

        let entry = get_fname_entry(self, entry_id)?;

        if entry.get_length() > 1024 {
            return Err(format!("Invalid string length: {}", entry.get_length()));
        }

        let length = u8::try_from(entry.get_length())
            .map_err(|e| format!("Failed to convert length to u8: {e}"))?;

        let decrypted_name = &entry.name[..entry.get_length() as usize]
            .iter()
            .enumerate()
            .map(|(i, c)| {
                c ^ length ^ ((self.game_engine.fname_decryption_key >> ((i & 3) * 8)) & 0xFF) as u8
            })
            .collect::<Vec<u8>>();

        let decrypted_name = String::from_utf8(decrypted_name.clone())
            .map_err(|e| format!("Failed to decrypt string: {e}"));

        match decrypted_name {
            Ok(name) => {
                FNAME_POOL_MAP
                    .lock()
                    .unwrap()
                    .insert(entry_id, name.clone());
                //
                Ok(name)
            }
            Err(e) => Err(e),
        }
    }
}

impl ReadGame for ValorantContext<'_, '_> {
    fn read<T>(&self, address: u64) -> T {
        if is_guarded_addr(address) {
            return self
                .mem_ctx
                .driver_ctx
                .read_physical_memory(self.guarded_memory_physical + (address & 0x00FF_FFFF));
        }
        self.mem_ctx
            .driver_ctx
            .read_virtual_memory(self.valorant, address)
    }

    fn read_into(&self, address: u64, out_buffer: *mut u8, size: usize) {
        if is_guarded_addr(address) {
            self.mem_ctx.driver_ctx.read_raw_physical_memory(
                self.guarded_memory_physical + (address & 0x00FF_FFFF),
                out_buffer,
                size,
            );
        }
        self.mem_ctx
            .driver_ctx
            .read_virtual_memory_into(self.valorant, address, out_buffer, size);
    }
}

pub const fn is_guarded_addr(addr: u64) -> bool {
    let filter = 0xFFFF_FFF0_0000_0000;
    let result = addr & filter;
    result == 0x0080_0000_0000 || result == 0x0100_0000_0000
}

pub fn get_guarded_memory_address(
    mem_ctx: &MemoryContext,
    bigpool_table: u64,
    bigpool_table_size: u64,
) -> Result<u64, ()> {
    let kernel_process = ProcessContext {
        eprocess: 0,
        directory_table_base: mem_ctx.kernel_ctx.cr3_phys,
        process_id: 4,
        base_addr: mem_ctx.kernel_ctx.base_virtual,
        nt_header: mem_ctx.kernel_ctx.nt_header,
        dos_header: mem_ctx.kernel_ctx.dos_header,
        image_section_header_ptr: 0,
    };

    // Enumerate the bigpool table
    loop {
        //  for i in 0..min(bigpool_table_size, 0x10000) {
        for i in 0..bigpool_table_size {
            let pool_entry: PoolTrackerBigPages<[u8; 0x18]> = mem_ctx
                .driver_ctx
                .read_virtual_memory(&kernel_process, bigpool_table + (i * 0x18));
            if pool_entry.number_of_bytes() != 0x200000 {
                continue;
            }
            log::info!("Pool Entry VA: 0x{:X}", pool_entry.va());
            log::info!("Pool Entry Size: 0x{:X}", pool_entry.number_of_bytes());
            // Check if u64 at uworld offset returns a guarded address
            let uworld_addr = pool_entry.va() + offsets::UWORLD;
            let uworld: u64 = mem_ctx
                .driver_ctx
                .read_virtual_memory(&kernel_process, uworld_addr);
            log::info!("UWorld: 0x{:X}", uworld);
            if is_guarded_addr(uworld) {
                return Ok(pool_entry.va());
            }
        }
        log::warn!("Failed to find guarded memory address, retrying");
        // std::thread::sleep(Duration::from_millis(500));
    }
}

pub fn get_fname_entry(val_ctx: &ValorantContext, entry_id: u32) -> Result<FNameEntry, String> {
    let chunk_offset = entry_id >> 16;
    let name_offset = u64::from(entry_id & 0xFFFF);

    let name_pool_chunk_addr =
        val_ctx.valorant.base_addr + offsets::FNAME_POOL + u64::from((chunk_offset + 2) * 8);

    let name_pool_chunk = val_ctx.read::<u64>(name_pool_chunk_addr);

    // If the name_offset is too big, do not read the entry
    if name_offset > 0xFFFF {
        return Err(format!(
            "Invalid name offset: {name_offset} (entry_id: {entry_id})"
        ));
    }

    if name_pool_chunk > u64::MAX - (4 * name_offset) {
        return Err(format!("Overflow for chunk of id {entry_id}"));
    }
    let entry_offset = name_pool_chunk + (4 * name_offset);
    let entry = val_ctx.read::<FNameEntry>(entry_offset);

    Ok(entry)
}
