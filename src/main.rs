//#![feature(proc_macro_hygiene, decl_macro)]

use crate::driver::{EProcessOffsets, MemoryContext, ProcessContext};
use dotenv::dotenv;
mod driver;
mod utils;
mod valorant;
mod webserver;

extern crate log;

#[tokio::main]
async fn main() {
    dotenv().ok();
    use env_logger::{Builder, Target};
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.filter_level(log::LevelFilter::Info).init();
    log::info!("Starting up");

    let driver_ctx = driver::init();
    if driver_ctx.is_none() {
        log::error!("Driver init failed");
        return;
    }
    log::info!("Initial Setup Success, finding kernel DirectoryTableBase");
    let driver_ctx = driver_ctx.unwrap();
    let halp_low_stub = utils::find_halp_low_stub(&driver_ctx); // Returns (va,pa)
    if halp_low_stub.is_none() {
        log::error!("Failed to find DirectoryTableBase");
        return;
    }
    let (kernel_cr3_va, kernel_cr3_pa) = halp_low_stub.unwrap();
    log::info!(
        "DTB Addresses, Virtual : 0x{:x}, Physical: 0x{:x}",
        kernel_cr3_va,
        kernel_cr3_pa
    );

    log::info!("Attempting to find ntoskernel");
    let kern_ctx = utils::get_kernel_base(&driver_ctx, kernel_cr3_pa, kernel_cr3_va);
    let mut mem_ctx = MemoryContext {
        driver_ctx,
        kernel_ctx: kern_ctx,
        process_ctx: ProcessContext::default(),
        shadow_memory_physical_address: 0,
        eproc_ctx: EProcessOffsets::default(),
    };

    let module_to_dump = "VALORANT-Win64";
    log::info!("Attempting to find {}", module_to_dump);

    let dump_process = utils::get_process(&mem_ctx, module_to_dump, module_to_dump);
    if dump_process.is_err() {
        log::error!("Failed to find {}", module_to_dump);
        return;
    }
    let dump_process = dump_process.unwrap();

    log::info!(
        "{} CR3 : {:X}",
        module_to_dump,
        dump_process.directory_table_base
    );
    log::info!("{} Base : {:X}", module_to_dump, dump_process.base_addr);

    utils::dump_module(
        &mem_ctx,
        &dump_process,
        dump_process.base_addr,
        module_to_dump,
    );

    log::info!("Looking for BPT");
    let bigpool_table = utils::get_big_pool_table(&mem_ctx);
    if bigpool_table.is_err() {
        log::error!("Failed to find BPT");
        return;
    }
    let bigpool_table = bigpool_table.unwrap();
    log::info!("BPT: {:x?}", bigpool_table);
    log::info!("Starting Webserver");

    let _ = std::thread::spawn(webserver::index::initialize);

    // We should probably change this to initialize valorant first.
    valorant::init(&mut mem_ctx, bigpool_table.0, bigpool_table.1);
}
