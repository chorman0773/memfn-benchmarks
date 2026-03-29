use core::cmp::Ordering;
use core::ffi::{c_int, c_void};
use core::mem::MaybeUninit;

macro_rules! memcpy_asm {
    ($dom_size:literal, $dom_copy_instr:literal, $dom_copy_reg:literal, $(($xymm_size:literal, $xymm_sub_copy_instr:literal, $xymm_subcopy_reg:literal)),* $(end $end_inst:literal)?) => {
        core::arch::naked_asm! {
            "mov rax, rdi",
            "cmp rdx, {domsize}",
            "jb 3f",
            "2:",
            ::core::concat!($dom_copy_instr, " ", $dom_copy_reg, ", [rsi]"),
            ::core::concat!($dom_copy_instr, " [rdi], ", $dom_copy_reg),
            "lea rsi, [rsi+{domsize}]",
            "lea rdi, [rdi+{domsize}]",
            "lea rdx, [rdx-{domsize}]",
            "cmp rdx, {domsize}",
            "jae 2b",
            "3:",
            $(
                ::core::concat!("cmp rdx, ", ::core::stringify!($xymm_size)),
                "jb 2f",
                ::core::concat!($xymm_sub_copy_instr, " ", $xymm_subcopy_reg, ", [rsi]"),
                ::core::concat!($xymm_sub_copy_instr, " [rdi], ", $xymm_subcopy_reg),
                ::core::concat!("lea rsi, [rsi+", ::core::stringify!($xymm_size), "]"),
                ::core::concat!("lea rdi, [rdi+", ::core::stringify!($xymm_size), "]"),
                ::core::concat!("lea rdx, [rdx-", ::core::stringify!($xymm_size), "]"),
                "2:",
            )*
            "cmp rdx, 8",
            "jb 2f",
            "mov rcx, [rsi]",
            "mov [rdi], rcx",
            "lea rsi, [rsi+8]",
            "lea rdi, [rdi+8]",
            "lea rdx, [rdx-8]",
            "2:",
            "cmp rdx, 4",
            "jb 2f",
            "mov ecx, [rsi]",
            "mov [rdi], ecx",
            "lea rsi, [rsi+4]",
            "lea rdi, [rdi+4]",
            "lea rdx, [rdx-4]",
            "2:",
            "cmp rdx, 2",
            "jb 2f",
            "mov cx, [rsi]",
            "mov [rdi], cx",
            "2:",
            "test rdx, rdx",
            "je 2f",
            "mov cl, [rsi]",
            "mov [rdi], cl",
            "2:",
            "xor ecx, ecx",
            $($end_inst,)?
            "ret",
            domsize = const $dom_size,
        }
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memcpy_avx(
    dest: *mut c_void,
    src: *const c_void,
    len: usize,
) -> *mut c_void {
    memcpy_asm! {
        32, "vmovups", "ymm0", (16, "vmovups", "xmm0") end "vzeroupper"
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memcpy_avx512(
    dest: *mut c_void,
    src: *const c_void,
    len: usize,
) -> *mut c_void {
    memcpy_asm! {
        64, "vmovups", "zmm0", (32, "vmovups", "ymm0"), (16, "vmovups", "xmm0") end "vzeroupper"
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memcpy_sse(
    dest: *mut c_void,
    src: *const c_void,
    len: usize,
) -> *mut c_void {
    memcpy_asm! {
        16, "movups", "xmm0",
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memcpy_erms(
    dest: *mut c_void,
    src: *const c_void,
    len: usize,
) -> *mut c_void {
    core::arch::naked_asm! {
        "mov rax, rdi",
        "mov rcx, rdx",
        "test rcx, rcx",
        "je 2f",
        "rep movsb",
        "2:",
        "ret",
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memmove_erms(
    dest: *mut c_void,
    src: *const c_void,
    len: usize,
) -> *mut c_void {
    core::arch::naked_asm! {
        "mov rax, rdi",
        "mov rcx, rdx",
        "test rcx, rcx",
        "je 2f",
        "rep movsb",
        "2:",
        "ret",
    }
}

macro_rules! memset_vector_asm {
    ($dom_size:literal, $dom_store_instr:literal, $dom_reg:literal, $shuffle_size:literal,
            $($preamble:literal),*
            $(($xymm_size:literal, $xymm_sub_store_instr:literal, $xymm_sub_reg:literal)),* $(end $($end_inst:literal),+)?) => {
        core::arch::naked_asm! {
            $($preamble,)*
            "cmp rdx, {domsize}",
            "jb 3f",
            "2:",
            ::core::concat!($dom_store_instr, "[rdi], ", $dom_reg),
            "lea rdi, [rdi+{domsize}]",
            "lea rdx, [rdx-{domsize}]",
            "cmp rdx, {domsize}",
            "jae 2b",
            "3:",
            $(
                ::core::concat!("cmp rdx, ", ::core::stringify!($xymm_size)),
                "jb 2f",
                ::core::concat!($xymm_sub_store_instr, " [rdi], ", $xymm_sub_reg),
                ::core::concat!("lea rdi, [rdi+", ::core::stringify!($xymm_size), "]"),
                ::core::concat!("lea rdx, [rdx-", ::core::stringify!($xymm_size), "]"),
                "2:",
            )*
            "cmp rdx, 8",
            "jb 2f",
            "mov [rdi], rcx",
            "lea rdi, [rdi+8]",
            "lea rdx, [rdx-8]",
            "2:",
            "cmp rdx, 4",
            "jb 2f",
            "mov [rdi], ecx",
            "lea rdi, [rdi+4]",
            "lea rdx, [rdx-4]",
            "2:",
            "cmp rdx, 2",
            "jb 2f",
            "mov [rdi], cx",
            "lea rdi, [rdi+2]",
            "lea rdx, [rdx-2]",
            "2:",
            "test rdx, rdx",
            "je 2f",
            "mov [rdi], cl",
            "2:",
            $($($end_inst,)+)?
            "ret",
            ".align {shufflesize}",
            "4:",
            ".space {shufflesize}",
            domsize = const $dom_size,
            shufflesize = const $shuffle_size,
        }
    };
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memset_sse4(dest: *mut c_void, val: c_int, len: usize) -> *mut c_void {
    memset_vector_asm!(
        16,
        "movdqu",
        "xmm0",
        16,
        "movd xmm0, rsi",
        "pshufb xmm0, [4f+rip]",
        "movq rcx, xmm0"
    )
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memset_avx(dest: *mut c_void, val: c_int, len: usize) -> *mut c_void {
    memset_vector_asm! (32, "vmovdqu", "ymm0", 32,
        "vmovd xmm0, rsi",
        "vpshufb ymm0, ymm0, [4f+rip]",
        "vmovq rcx, xmm0"
        (16, "vmovdqu", "xmm0")
        end "vzeroupper"
    )
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memset_avx512(dest: *mut c_void, val: c_int, len: usize) -> *mut c_void {
    memset_vector_asm! (64, "vmovdqu64", "zmm0", 64,
        "vmovd xmm0, rsi",
        "vpshufb zmm0, zmm0, [4f+rip]",
        "vmovq rcx, xmm0"
        (32, "vmovdqu", "ymm0"),
        (16, "vmovdqu", "xmm0")
        end "vzeroupper"
    )
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memset_erms(dest: *mut c_void, val: c_int, len: usize) -> *mut c_void {
    core::arch::naked_asm! {
        "mov rcx, rdx",
        "mov rax, rdi", // save the register, but we need to use rax
        "xchg rax, rsi",
        "test rcx, rcx",
        "je 2f",
        "rep stosb",
        "2:",
        "mov rax, rsi",
        "ret"
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memchr_erms(
    src: *const c_void,
    val: c_int,
    len: usize,
) -> *const c_void {
    core::arch::naked_asm! {
        "mov rcx, rdx",
        "mov eax, esi",
        "test rcx, rcx",
        "je 2f",
        "repz scasb",
        "mov rax, rdi",
        "ret",
        "2:",
        "xor eax, eax",
        "ret",
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn __memcmp_erms(
    src1: *const c_void,
    src2: *const c_void,
    len: usize,
) -> c_int {
    core::arch::naked_asm! {
        "mov rcx, rdx",
        "test rcx, rcx",
        "je 2f",
        "repz cmpsb",
    }
}

macro_rules! copy_as_type_unaligned {
    ($ty:ty, $dest:ident, $src:ident, $len:ident) => {
        #[allow(unused_assignments)]
        {
            $dest
                .cast::<MaybeUninit<$ty>>()
                .write_unaligned($src.cast::<MaybeUninit<$ty>>().read_unaligned());
            $dest = $dest.byte_add(core::mem::size_of::<$ty>());
            $src = $src.byte_add(core::mem::size_of::<$ty>());
            $len -= core::mem::size_of::<$ty>();
        }
    };
}

pub unsafe extern "C" fn __memcpy_generic(
    dest: *mut c_void,
    mut src: *const c_void,
    mut len: usize,
) -> *mut c_void {
    let mut dest_ptr = dest;

    while len >= core::mem::size_of::<usize>() {
        unsafe {
            copy_as_type_unaligned!(usize, dest_ptr, src, len);
        }
    }

    #[cfg(false)]
    if len >= 8 {
        unsafe {
            copy_as_type_unaligned!(u64, dest_ptr, src, len);
        }
    }

    #[cfg(any(target_pointer_width = "64", false))]
    if len >= 4 {
        unsafe {
            copy_as_type_unaligned!(u32, dest_ptr, src, len);
        }
    }

    if len >= 2 {
        unsafe {
            copy_as_type_unaligned!(u16, dest_ptr, src, len);
        }
    }

    if len == 1 {
        unsafe {
            copy_as_type_unaligned!(u8, dest_ptr, src, len);
        }
    }
    dest
}

pub unsafe extern "C" fn __memmove_generic(
    dest: *mut c_void,
    src: *const c_void,
    len: usize,
) -> *mut c_void {
    let mut dest_ptr = dest.cast::<MaybeUninit<u8>>();
    let mut src_ptr = src.cast::<MaybeUninit<u8>>();

    let end = unsafe { dest_ptr.add(len) };

    while dest_ptr != end {
        unsafe {
            dest_ptr.write(src_ptr.read());
        }
        unsafe {
            dest_ptr = dest_ptr.add(1);
        }
        unsafe {
            src_ptr = src_ptr.add(1);
        }
    }

    dest
}

pub unsafe extern "C" fn __memset_generic(
    dest: *mut c_void,
    val: c_int,
    len: usize,
) -> *mut c_void {
    let byte = val as u8;
    let mut dest_ptr = dest.cast::<u8>();

    let end = unsafe { dest_ptr.add(len) };

    while dest_ptr != end {
        unsafe {
            dest_ptr.write(byte);
        }
        unsafe {
            dest_ptr = dest_ptr.add(1);
        }
    }

    dest
}

pub unsafe extern "C" fn __memchr_generic(
    src: *const c_void,
    val: c_int,
    len: usize,
) -> *const c_void {
    let mut src = src.cast::<u8>();
    let end = src.wrapping_add(len);
    let val = val as u8;

    while src != end {
        if unsafe { src.read() } == val {
            return src.cast();
        }
        src = unsafe { src.add(1) };
    }

    core::ptr::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __memcmp_generic(
    src1: *const c_void,
    src2: *const c_void,
    len: usize,
) -> c_int {
    let s1 = src1.cast::<u8>();
    let s2 = src2.cast::<u8>();

    let mut pos = 0;

    while pos < len {
        match unsafe { s1.add(pos).read().cmp(&s2.add(pos).read()) } {
            Ordering::Equal => {
                pos += 1;
            }
            val => return val as c_int,
        }
    }

    0
}
