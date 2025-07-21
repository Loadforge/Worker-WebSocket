use sysinfo::System;

pub fn get_hardware_info() -> (u64, u64, u64) {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_cores = sys.cpus().len() as u64;  
    let total_mem = sys.total_memory();
    let free_mem = sys.available_memory();

    (cpu_cores, total_mem, free_mem)
}
