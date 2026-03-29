#![feature(macro_metavar_expr_concat)]

mod impls;

mod detect;

/// Safety:
/// Given an equivalent value for `self`, two consecutive calls to [`BufferSupplier::get`] shall return distinct buffers with the same length
pub unsafe trait BufferSupplier {
    fn get(&self) -> Box<[u8]>;

    fn get_single_byte(&self) -> u8;
}

unsafe impl<B: BufferSupplier + ?Sized> BufferSupplier for &B {
    fn get(&self) -> Box<[u8]> {
        <B as BufferSupplier>::get(self)
    }

    fn get_single_byte(&self) -> u8 {
        <B as BufferSupplier>::get_single_byte(self)
    }
}

fn zeroed_slice(len: usize) -> Box<[u8]> {
    unsafe { Box::new_zeroed_slice(len).assume_init() }
}

pub struct Zeroed(usize);

unsafe impl BufferSupplier for Zeroed {
    fn get(&self) -> Box<[u8]> {
        zeroed_slice(self.0)
    }

    fn get_single_byte(&self) -> u8 {
        0
    }
}

pub struct Random(usize);

unsafe impl BufferSupplier for Random {
    fn get(&self) -> Box<[u8]> {
        let mut buffer = zeroed_slice(self.0);

        getrandom::fill(&mut buffer).unwrap();

        buffer
    }

    fn get_single_byte(&self) -> u8 {
        let mut b = 0u8;
        getrandom::fill(core::slice::from_mut(&mut b)).unwrap();

        b
    }
}

pub fn rdtsc() -> u64 {
    let mut _ignored: u64;
    let mut lo: u64;
    let mut hi: u64;
    unsafe {
        core::arch::asm!("xchg rbx, {rbx}",
            "cpuid",
            "rdtsc",
            "mov {lo:e}, eax",
            "mov {hi:e}, edx",
            "cpuid",
            "xchg rbx, {rbx}",
            rbx = out(reg) _ignored,
            lo = out(reg) lo,
            hi = out(reg) hi,
            inout("eax") 0=>_,
            out("edx") _,
            out("rcx") _,

        );
    }

    lo | (hi << 32)
}

macro_rules! def_timing_funcs{
    {
        |$buffer_supplier:ident, $func_name:ident|
        $($fn_name:ident [
            $($(#[target_feature($(enable = $feature_name:tt),+)])|+ $impl_name:ident),*
            $(,)?
        ] $init_code:block |$($init_outputs:ident),*| $call_routine:block)*
    } => {
        $(
            fn ${concat(bench_, $fn_name)}  <F: BufferSupplier>($buffer_supplier: F, name: &str) -> Option<u64> {
                let func = match name {
                    $(::core::stringify!($impl_name) if $($($crate::is_x86_feature_detected!($feature_name))&&+)||* => {
                        impls::${concat(__, $fn_name, _, $impl_name)}
                    })*

                    "generic" => impls::${concat(__, $fn_name, _generic)},
                    _ => return None
                };

                let $func_name = core::hint::black_box(func);

                #[allow(unused_mut)]
                let ($(mut $init_outputs,)*) = core::hint::black_box($init_code);

                let begin = rdtsc();

                $call_routine

                let end = rdtsc();

                Some(end - begin)

            }
        )*

        pub fn bench_entry<F: BufferSupplier>(buffer: F, to_test: &[&str], iters: usize) {
            $(
                println!("{}:", ::core::stringify!($fn_name));

                'a: for test in to_test {
                    let mut total_time = 0u128;
                    let mut max_time = 0;
                    let mut min_time = u64::MAX;
                    for _ in 0..iters {
                        let Some(time) = ${concat(bench_, $fn_name)}(&buffer, test) else {
                            continue 'a;
                        };
                        total_time += time as u128;

                        max_time = max_time.max(time);
                        min_time = min_time.min(time);
                    }

                    let mean = total_time as f64 / iters as f64;
                    println!("\t{test}: Mean {mean:.3} ticks, Max {max_time} ticks, Min {min_time} ticks");
                }
            )*
        }
    };
}

def_timing_funcs! {
    |supplier, func| memcpy [
        #[target_feature(enable = "sse2")] sse,
        #[target_feature(enable = "avx")] avx,
        #[target_feature(enable = "avx512f")] avx512,
        #[target_feature(enable = "erms")] erms
    ] {
        let dest = supplier.get();
        let src = supplier.get();
        (dest, src)
    } |dest, src| { unsafe { func(dest.as_mut_ptr().cast(), src.as_ptr().cast(), src.len()); } }

    memset [
        #[target_feature(enable = "sse3")] sse4,
        #[target_feature(enable = "avx")] avx,
        #[target_feature(enable = "avx512f", enable = "avx512bw")] avx512,
        #[target_feature(enable = "erms")] erms
    ] {
        let dest = supplier.get();
        let val = supplier.get_single_byte() as i32;
        (dest, val)
    } |dest, val| {
        unsafe { func(dest.as_mut_ptr().cast(), val, dest.len()); }
    }
}

fn main() {
    println!("X86 Feature List = {:#X?}", detect::__get_feature_list());
    dbg!(crate::is_x86_feature_detected!("avx512f"));
    let arr = ["sse", "avx", "avx512", "erms", "generic"];
    for (size, count) in [
        (8, 16384),
        (16, 16384),
        (32, 8192),
        (64, 8192),
        (128, 8192),
        (1024, 4096),
        (1024 * 1024, 4096),
        (257, 8192),
        (65535, 2048),
        (68, 8192),
    ] {
        println!("Zeroed Buffer ({size} bytes):");
        bench_entry(Zeroed(size), &arr, count);
        println!("Randomized Buffer ({size} bytes):");
        bench_entry(Random(size), &arr, count);
    }
}
