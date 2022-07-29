//! Process management syscalls

use lock_api::Mutex;
use xmas_elf::program;

use crate::mm::VirtAddr;
use crate::mm::page_table::PageTable;
use crate::mm::PhysAddr;
use crate::config::MAX_SYSCALL_NUM;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, current_user_token, get_status, get_syscall_times, get_already_run_time, mmap, munmap};
use crate::timer::get_time_us;
use crate::mm::address::VPNRange;
use crate::mm::KERNEL_SPACE;
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn get_physical_address(token:usize,ptr:*const u8) ->usize{
    let page_table= PageTable::from_token(token);
    let va = VirtAddr::from(ptr as usize);
    let ppn = page_table.find_pte(va.floor()).unwrap().ppn();
    PhysAddr::from(ppn).0 +va.page_offset() 
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let real = get_physical_address(current_user_token(), _ts as *const u8) as *mut TimeVal;
    unsafe {
        *real = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if (_port&!0x7 !=0)||(_port&0x7==0){
        return -1
    }
    let va = VirtAddr::from(_start);
    if va.aligned(){
        return mmap(_start, _len, _port);
    }else{
        return -1; 
    }
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    let va = VirtAddr::from(_start);
    if va.aligned(){
        return munmap(_start, _len);
    }else{
        return -1;
    }
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let real = get_physical_address(current_user_token(), ti as *const u8) as *mut TaskInfo;
    unsafe {
        *real = TaskInfo{
            status:get_status(),
            syscall_times:get_syscall_times(),
            time:get_already_run_time()/1000,
        }
    }
    0
}
