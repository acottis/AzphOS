/// Whether or not floats are used. This is used by the MSVC calling convention
/// and it just has to exist.
#[export_name="_fltused"]
pub static FLTUSED: usize = 0;


/// libc `memmove` implementation in Rust
///
/// # Parameters
///
/// * `dest` - Pointer to memory to copy to
/// * `src`  - Pointer to memory to copy from
/// * `n`    - Number of bytes to copy
///
#[no_mangle]
pub unsafe extern fn memmove(dest: *mut u8, src: *const u8, n: usize)
        -> *mut u8 {
    if src < dest as *const u8 {
        // copy backwards
        let mut ii = n;
        while ii != 0 {
            ii -= 1;
            *dest.offset(ii as isize) = *src.offset(ii as isize);
        }
    } else {
        // copy forwards
        let mut ii = 0;
        while ii < n {
            *dest.offset(ii as isize) = *src.offset(ii as isize);
            ii += 1;
        }
    }

    dest
}
/// libc `memcpy` implementation in Rust
///
/// This implementation of `memcpy` is overlap safe, making it technically
/// `memmove`.
///
/// # Parameters
///
/// * `dest` - Pointer to memory to copy to
/// * `src`  - Pointer to memory to copy from
/// * `n`    - Number of bytes to copy
///
#[no_mangle]
pub unsafe extern fn memcpy(dest: *mut u8, src: *const u8, n: usize)
        -> *mut u8 {
    memmove(dest, src, n)
}
/// libc `memcmp` implementation in Rust
///
/// # Parameters
///
/// * `s1` - Pointer to memory to compare with s2
/// * `s2` - Pointer to memory to compare with s1
/// * `n`  - Number of bytes to set
#[no_mangle]
unsafe extern fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut ii = 0;
    while ii < n {
        let a = *s1.offset(ii as isize);
        let b = *s2.offset(ii as isize);
        if a != b {
            return a as i32 - b as i32
        }
        ii += 1;
    }
    0
}

/// libc `memset` implementation in Rust
///
/// # Parameters
///
/// * `s` - Pointer to memory to set
/// * `c` - Character to set `n` bytes in `s` to
/// * `n` - Number of bytes to set
///
#[no_mangle]
unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.offset(i as isize) = c as u8;
        i += 1;
    }
    s
}

#[no_mangle]
unsafe extern fn _aulldiv(){
    panic!();
}

#[no_mangle]
unsafe extern fn _aullrem(){
    panic!();
}