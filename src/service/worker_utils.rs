use crate::{
    model::{computing_result::ComputingResult, result_with_worker::ResultWithLoad, worker::Load},
    service::sysinfo_utils::{get_cpu_usage, get_memory_usage},
};

pub fn wrap_result_with_worker_load(result: Option<ComputingResult>) -> ResultWithLoad {
    ResultWithLoad::new(result, Load::new(get_cpu_usage(), get_memory_usage()))
}
