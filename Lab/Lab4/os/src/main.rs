#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
//#[macro_use] is on the last position
#[macro_use]
//mod console is ahead of panic and sbi
mod console;
mod panic;
mod sbi;
mod interrupt;
mod memory;
mod process;
extern crate alloc;
use process::*;
use alloc::sync::Arc;
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    memory::init();
    interrupt::init();

    {
        let mut processor = PROCESSOR.lock();
        // 创建一个内核进程
        let kernel_process = Process::new_kernel().unwrap();
        // 为这个进程创建多个线程，并设置入口均为 sample_process，而参数不同
        for i in 1..9usize {
            processor.add_thread(create_kernel_thread(
                kernel_process.clone(),
                sample_process as usize,
                Some(&[i]),
            ));
        }
    }

    extern "C" {
        fn _restore(context: usize);
    }
    // 获取第一个线程的 Context
    let context = PROCESSOR.lock().prepare_next_thread();
    // 启动第一个线程
    unsafe { _restore(context as usize) };
    unreachable!()
}

fn sample_process(id: usize) {
    println!("hello from kernel thread {}", id);
}
/// 内核线程需要调用这个函数来退出

fn kernel_thread_exit() {
    // 当前线程标记为结束
    PROCESSOR.lock().current_thread().as_ref().inner().dead = true;
    // 制造一个中断来交给操作系统处理
    unsafe { llvm_asm!("ebreak" :::: "volatile") };
}

/// 创建一个内核进程
pub fn create_kernel_thread(
    process: Arc<Process>,
    entry_point: usize,
    arguments: Option<&[usize]>,
) -> Arc<Thread> {
    // 创建线程
    let thread = Thread::new(process, entry_point, arguments).unwrap();
    // 设置线程的返回地址为 kernel_thread_exit
    thread
        .as_ref()
        .inner()
        .context
        .as_mut()
        .unwrap()
        .set_ra(kernel_thread_exit as usize);

    thread
}    
  
