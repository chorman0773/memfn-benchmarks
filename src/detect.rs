use std::{
    arch::x86_64::{__cpuid, __cpuid_count},
    sync::LazyLock,
};

// cpuid[eax=1].ecx
// cpuid[eax=1].edx
// cpuid[eax=7,ecx=0].ecx
// cpuid[eax=7,ecx=0].edx
// cpuid[eax=7,ecx=0].ebx
// cpuid[eax=7,ecx=1].eax
// cpuid[eax=7,ecx=1].ecx
// cpuid[eax=7,ecx=1].edx
// cpuid[eax=7,ecx=1].ebx
// cpuid[eax=7,ecx=2].eax
// cpuid[eax=7,ecx=2].ecx
// cpuid[eax=7,ecx=2].edx
// Reserved
// Reserved
// cpuid[eax=0x80000001].ecx*
// cpuid[eax=0x80000001].edx
// cpuid[eax=0x0D,ecx=0].eax
// cpuid[eax=0x0D,ecx=0].edx
// cpuid[eax=0x0D,ecx=1].eax
// Reserved
// cpuid[eax=0x24,ecx=0].ebx
// cpuid[eax=0x24,ecx=1].ecx
// Reserved..

const fn select_mask(sel: bool, mask: u32) -> u32 {
    if sel { u32::MAX } else { !mask }
}

static FEATURE_LIST: LazyLock<[u32; 36]> = LazyLock::new(|| {
    let cpuid_eax1 = __cpuid(1);
    let cpuid_eax7_ecx0 = __cpuid_count(7, 0);
    let cpuid_eax7_ecx1 = if cpuid_eax7_ecx0.eax >= 1 {
        __cpuid_count(7, 1)
    } else {
        unsafe { core::mem::zeroed() }
    };
    let cpuid_eax7_ecx2 = if cpuid_eax7_ecx0.eax >= 2 {
        __cpuid_count(7, 2)
    } else {
        unsafe { core::mem::zeroed() }
    };
    let cpuidx_eaxe1 = __cpuid(0x80000001);
    let cpuid_eaxD_ecx0 = __cpuid_count(0xD, 0);
    let cpuid_eaxD_ecx1 = __cpuid_count(0x0D, 1);
    let cpuid_eax24_ecx0 = __cpuid_count(0x24, 0);
    let cpuid_eax24_ecx1 = __cpuid_count(0x24, 1);

    let enable_osxsave = (cpuid_eax1.ecx & (1 << 27)) != 0;

    let getbv = if enable_osxsave {
        let lo: u64;
        let hi: u64;
        unsafe { core::arch::asm!("xgetbv", in("rcx") 0, out("rax") lo, out("rdx") hi) };

        (hi << 32) | lo
    } else {
        0
    };

    let use_avx512 = (getbv & (0b111 << 5)) == (0b111 << 5);
    let use_avx = (getbv & (1 << 2)) != 0;
    let use_apx = (getbv & (1 << 19)) != 0;

    let avx_mask_eax1_ecx = (1 << 28);
    let avx_mask_eax7_ec0_ebx = (1 << 5);

    let apx_mask_eax7_ecx1_edx = (1 << 21);

    let avx512_mask_eax7_ecx0_ecx = (1 << 1) | (1 << 6) | (1 << 11) | (1 << 12) | (1 << 14);
    let avx512_mask_eax7_ecx0_edx = (1 << 2) | (1 << 3) | (1 << 8) | (1 << 23);
    let avx512_mask_eax7_ecx0_ebx =
        (1 << 16) | (1 << 17) | (1 << 21) | (1 << 26) | (1 << 28) | (1 << 30) | (1 << 31);
    let avx512_mask_eax7_ecx1_eax = (1 << 5);

    let arr = [
        cpuid_eax1.ecx & select_mask(use_avx, avx_mask_eax1_ecx),
        cpuid_eax1.edx,
        cpuid_eax7_ecx0.ecx & select_mask(use_avx512, avx512_mask_eax7_ecx0_ecx),
        cpuid_eax7_ecx0.edx & select_mask(use_avx512, avx512_mask_eax7_ecx0_edx),
        cpuid_eax7_ecx0.ebx
            & select_mask(use_avx, avx_mask_eax7_ec0_ebx)
            & select_mask(use_avx512, avx512_mask_eax7_ecx0_ebx),
        cpuid_eax7_ecx1.eax & select_mask(use_avx512, avx512_mask_eax7_ecx1_eax),
        cpuid_eax7_ecx1.ecx,
        cpuid_eax7_ecx1.edx & select_mask(use_apx, apx_mask_eax7_ecx1_edx),
        cpuid_eax7_ecx1.ebx,
        cpuid_eax7_ecx2.eax,
        cpuid_eax7_ecx2.ecx,
        cpuid_eax7_ecx2.edx,
        0,
        0,
        cpuidx_eaxe1.ecx,
        cpuidx_eaxe1.edx,
        cpuid_eaxD_ecx0.eax,
        cpuid_eaxD_ecx0.edx,
        cpuid_eaxD_ecx1.eax,
        0,
        cpuid_eax24_ecx0.ebx,
        cpuid_eax24_ecx1.ecx,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];

    arr
});

#[doc(hidden)]
pub fn __get_feature_list() -> &'static [u32; 36] {
    &FEATURE_LIST
}

#[doc(hidden)]
#[macro_export]
macro_rules! __x86_feature_name_to_bit {
    ("x87") => {
        (1, 0)
    };
    ("sse") => {
        (1, 25)
    };
    ("sse2") => {
        (1, 26)
    };
    ("sse3") => {
        (0, 0)
    };
    ("ssse3") => {
        (0, 9)
    };
    ("sse4.1") => {
        (0, 19)
    };
    ("sse4.2") => {
        (0, 20)
    };
    ("xsave") => {
        (0, 27)
    };
    ("avx") => {
        (0, 28)
    };
    ("f16c") => {
        (0, 29)
    };
    ("rdrand") => {
        (0, 30)
    };
    ("avx2") => {
        (4, 5)
    };
    ("fsgsbase") => {
        (4, 0)
    };
    ("avx512f") => {
        (4, 16)
    };
    ("erms") => {
        (4, 9)
    };
    ("fsrm") => {
        (3, 4)
    };
    ("avx10") => {
        (8, 19)
    };
    ("cx8") => {
        (1, 8)
    };
    ("cmpxchg8b") => {
        (1, 8)
    };
    ("cx16") => {
        (0, 13)
    };
    ("cmpxchg16b") => {
        (0, 13)
    };
    ("avx512dq") => {
        (4, 17)
    };
    ("avx512bw") => {
        (4, 30)
    };
    ("avx512vl") => {
        (4, 31)
    };

    ($lit:literal) => {
        ::core::compile_error!(::core::concat!("Unknown x86 feature ", $lit))
    };
}

#[macro_export]
macro_rules! is_x86_feature_enabled {
    ($feature:tt) => {
        const {
            let _ = $crate::__x86_feature_name_to_bit!($feature);
            #[allow(unexpected_cfgs)]
            // We might use feature names that rustc doesn't like. We validate the names above
            let __val = ::core::cfg!(target_feature = $feature);
            __val
        }
    };
    ($feature:tt,) => {
        $crate::is_x86_feature_enabled!($feature)
    };
}

#[macro_export]
macro_rules! is_x86_feature_detected {
    ($feature:tt) => {
        $crate::is_x86_feature_enabled!($feature) || {
            let (index, bit) = $crate::__x86_feature_name_to_bit!($feature);
            let arr = $crate::detect::__get_feature_list();

            ((arr[index] & (1 << bit)) != 0)
        }
    };
}
