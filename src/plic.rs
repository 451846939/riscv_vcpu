use core::cell::OnceCell;
use core::num::{NonZero, NonZeroU32};
use lazyinit::LazyInit;
use memory_addr::{pa, PhysAddr};
use riscv_plic::{HartContext, InterruptSource, Plic};
use axvisor_api::arch;
use axvisor_api::memory::phys_to_virt;
use axvisor_api::vmm::current_vcpu_id;

#[derive(Clone, Debug)]
struct Context {
    hart_id: usize,
}

impl HartContext for Context {
    fn index(self) -> usize {
        self.hart_id * 2 + 1
    }
}

impl Context {
    const fn new(hart_id: usize) -> Self {
        Self { hart_id }
    }
}

struct Interrupt {
    irq_num: usize,
}

impl InterruptSource for Interrupt {
    fn id(self) -> core::num::NonZeroU32 {
        core::num::NonZeroU32::new(self.irq_num as u32).unwrap()
    }
}


const PLIC_BASE: PhysAddr = pa!(0x0c000000);


pub static GLOBAL_PLIC: LazyInit<Plic> = LazyInit::new();

pub fn init_plic() {
    let plic_ptr = phys_to_virt(PLIC_BASE).as_mut_ptr();
    // let plic_ptr = 0x0c000000 as *mut u8;
    GLOBAL_PLIC.call_once(|| Plic::new(plic_ptr));
}
const CAUSE_INTR_SOFT: usize = 1;
const CAUSE_INTR_TIMER: usize = 5;
const CAUSE_INTR_EXTERNAL: usize = 9;

const SIP_SSIP: usize = 1 << 1;

pub const IRQ_IPI: usize = 60; // for clint, not plic
pub const IRQ_HYPERVISOR_TIMER: usize = 61; // for clint, not plic
pub const IRQ_GUEST_TIMER: usize = 62; // not valid

pub fn riscv_get_pending_irqs() -> Option<usize> {
    info!("riscv_get_pending_irqs");
    let ctx = Context::new(current_vcpu_id());

    let irq = GLOBAL_PLIC.claim(ctx.clone());
    match irq {
        Some(nz) if nz.get() != 0 => {
            let irq_num = nz.get() as usize;
            GLOBAL_PLIC.complete(ctx, Interrupt { irq_num });
            Some(irq_num)
        }
        _ => None,
    }
}
pub fn get_claim(hart_id: usize) -> Option<NonZeroU32> {
    let ctx = Context::new(hart_id);
    GLOBAL_PLIC.claim(ctx)
}

pub fn test_plic_pending(irq_num: usize) {
    let bool=GLOBAL_PLIC.is_pending(crate::plic::Interrupt{irq_num});
    info!("set_plic_pending: irq_num={}, is_pending={}", irq_num, bool);
    arch::inject_virtual_interrupt(64);
    // PLIC pending 寄存器物理基址
    // const PLIC_PENDING_BASE: usize = 0x0C00_1000;
    // let word_offset = irq_num / 32;
    // let bit_offset = irq_num % 32;
    // // let pa = pa!(PLIC_PENDING_BASE + word_offset * 4);
    // let pa = PLIC_PENDING_BASE + word_offset * 4;
    // // 用物理->虚拟地址转换
    // let reg = pa as *mut u32;
    // unsafe {
    //     let val = core::ptr::read_volatile(reg);
    //     core::ptr::write_volatile(reg, val | (1 << bit_offset));
    // }
}

