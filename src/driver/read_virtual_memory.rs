use super::{DriverContext, ProcessContext, ReadPhysicalMemory, ReadVirtualMemory};

impl ReadVirtualMemory for DriverContext {
    fn read_virtual_memory<T>(&self, process_ctx: &ProcessContext, addr: u64) -> T {
        let virt_memory_page = addr & !0xfffu64;
        let virt_memory_page_with_size = addr + std::mem::size_of::<T>() as u64;
        /*   log::info!(
            "Addr {:x} Page {:x} Final {:x} Size {:x}",
            addr,
            virt_memory_page,
            virt_memory_page_with_size,
            std::mem::size_of::<T>() as u64
        ); */
        let pages_to_read = (((virt_memory_page_with_size & !0xfffu64) - virt_memory_page)
            / 0x1000)
            + u64::from(virt_memory_page_with_size & 0xfff != 0);
        /*  log::info!(
            "Addr {:x} Page {:x} Final {:x} Size {:x} Pages {}",
            addr,
            virt_memory_page,
            virt_memory_page_with_size,
            std::mem::size_of::<T>() as u64,
            pages_to_read
        ); */
        if pages_to_read < 2 {
            let phys_addr = self.translate_linear_address(process_ctx.directory_table_base, addr);
            return self.read_physical_memory(phys_addr);
        }

        // If read is crossing page boundry
        let size_of_return_buffer = std::mem::size_of::<T>();
        let mut data_buffer = vec![0u8; size_of_return_buffer];
        let mut bytes_read = 0;
        /* log::info!(
            "Size of return buffer {:x} Pages {}",
            size_of_return_buffer,
            pages_to_read
        ); */
        for i in 0..pages_to_read {
            // Translate virtual address to physical address
            let phys_addr = if i == 0 {
                self.translate_linear_address(process_ctx.directory_table_base, addr)
            } else {
                self.translate_linear_address(
                    process_ctx.directory_table_base,
                    virt_memory_page + i * 0x1000,
                )
            };

            // Get how many bytes to read in current page
            let bytes_to_read_in_page = if i == 0 {
                let next_page = virt_memory_page + 0x1000;
                next_page - addr
            } else if size_of_return_buffer - bytes_read < 0x1000 {
                (size_of_return_buffer - bytes_read) as u64
            } else {
                0x1000u64
            };
            // log::info!("Page {} Bytes to read {:x}", i, bytes_to_read_in_page);
            let mut page_buffer = vec![0u8; bytes_to_read_in_page as usize];
            self.read_raw_physical_memory(
                phys_addr,
                page_buffer.as_mut_ptr(),
                bytes_to_read_in_page as usize,
            );

            // Copy bytes from page buffer to return buffer
            unsafe {
                std::ptr::copy(
                    page_buffer.as_ptr(),
                    data_buffer.as_mut_ptr().add(bytes_read),
                    bytes_to_read_in_page as usize,
                );
            }

            bytes_read += bytes_to_read_in_page as usize;
        }
        if bytes_read == size_of_return_buffer {
            //log::info!("Read Successful");
            // Can you believe it missing the return statement here caused me 2 hours of debugging
            unsafe { return std::ptr::read(data_buffer.as_ptr() as *const T) };
        }
        unsafe { std::mem::zeroed() }
    }
    fn read_virtual_memory_into(
        &self,
        process_ctx: &ProcessContext,
        addr: u64,
        out_buffer: *mut u8,
        size: usize,
    ) -> bool {
        let virt_memory_page = addr & !0xfffu64;
        let virt_memory_page_with_size = addr + size as u64;

        let pages_to_read = (((virt_memory_page_with_size & !0xfffu64) - virt_memory_page)
            / 0x1000)
            + u64::from(virt_memory_page_with_size & 0xfff != 0);

        if pages_to_read < 2 {
            let phys_addr = self.translate_linear_address(process_ctx.directory_table_base, addr);
            self.read_raw_physical_memory(phys_addr, out_buffer, size);
            return true;
        }

        // If read is crossing page boundry
        let mut data_buffer = vec![0u8; size];
        let mut bytes_read = 0;
        //log::info!("Size of return buffer {:x} Pages {}", size, pages_to_read);
        for i in 0..pages_to_read {
            // Translate virtual address to physical address
            let phys_addr = if i == 0 {
                self.translate_linear_address(process_ctx.directory_table_base, addr)
            } else {
                self.translate_linear_address(
                    process_ctx.directory_table_base,
                    virt_memory_page + i * 0x1000,
                )
            };

            // Get how many bytes to read in current page
            let bytes_to_read_in_page = if i == 0 {
                let next_page = virt_memory_page + 0x1000;
                next_page - addr
            } else if size - bytes_read < 0x1000 {
                (size - bytes_read) as u64
            } else {
                0x1000u64
            };
            // log::info!("Page {} Bytes to read {:x}", i, bytes_to_read_in_page);
            let mut page_buffer = vec![0u8; bytes_to_read_in_page as usize];
            self.read_raw_physical_memory(
                phys_addr,
                page_buffer.as_mut_ptr(),
                bytes_to_read_in_page as usize,
            );

            // Copy bytes from page buffer to return buffer
            unsafe {
                std::ptr::copy(
                    page_buffer.as_ptr(),
                    data_buffer.as_mut_ptr().byte_add(bytes_read),
                    bytes_to_read_in_page as usize,
                );
            }

            bytes_read += bytes_to_read_in_page as usize;
        }
        if bytes_read == size {
            //log::info!("Read Successful");
            // Can you believe it missing the return statement here caused me 2 hours of debugging
            unsafe { std::ptr::copy(data_buffer.as_ptr(), out_buffer, size) };
            return true;
        } else {
            //log::info!("Read Failed");
        }
        false
    }
}
