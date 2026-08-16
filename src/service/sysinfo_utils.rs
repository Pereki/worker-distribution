use sysinfo::System;

pub fn get_cpu_usage() -> f64 {
    let mut sys = System::new_all();

    sys.refresh_cpu_usage();

    sys.global_cpu_usage() as f64
}

pub fn get_memory_usage() -> f64 {
    let mut sys = System::new_all();

    sys.refresh_memory();
    let total_ram = sys.total_memory();
    let used_ram = sys.used_memory();

    (used_ram as f64 / total_ram as f64) * 100.0
}
