/// Transmute and copy to u8 array.
///
/// # Unsafe
/// Forces $targ to be represented as an arry of u8. $num_bytes is assumed to be the number of
/// bytes $targ occupies. $num_bytes must also be the same size as the $outbuf used. First param
/// is just for readability at use site.
///

#[macro_export]
macro_rules! transmute_to_u8_from {
    ($t:ty, $num_bytes:expr, $targ:expr, $outbuf: expr) => ({
            use std::mem::transmute;
            use std::ptr::copy_nonoverlapping;
            unsafe {
                let buf = transmute::<_, [u8; $num_bytes]>($targ);
                copy_nonoverlapping( (&buf).as_ptr(), $outbuf.as_mut_ptr(), $num_bytes );
            }
        });
}

/// Setup buffer for transmutation then call into macro.
///
/// The macro setups the boiler plate to call transmute_to_u8_from! macro. $fnc is a fn that exists
/// to call transmute macro with correct parameters for converting a specific type.
///
/// ```ignore
/// fn u16_to_u8(target: u16, output: &mut [u8]) {
///    transmute_to_u8_from!(u16, 2, target, output);
/// }
/// ```
/// Naming the function as x_to_u8 increases readability at use site. Having the target byte size
/// as last param doesn't.
/// ```ignore
/// do_transmute!(u16_to_u8, input, &mut buf, &mut buf_offset, 2);
/// ```

#[macro_export]
macro_rules! do_transmute {
    ($fnc:ident, $input:expr, $to_buf:expr, $offset:expr, $size:expr) => ({
            #[allow(unused_imports)]
            use util::{u16_to_u8, u32_to_u8, f32_to_u8, append_bytes};
            let mut transmutebuf: [u8; $size] = [0; $size];
            $fnc($input, &mut transmutebuf);
            *$offset = *$offset + append_bytes( &transmutebuf, $to_buf, *$offset);
        });
}

/// Helper function to convert u16 to a [u8; 2].
pub fn u16_to_u8(target: u16, output: &mut [u8]) {
    transmute_to_u8_from!(u16, 2, target, output);
}
/// Helper function to convert u32 to a [u8; 4].
pub fn u32_to_u8(target: u32, output: &mut [u8]) {
    transmute_to_u8_from!(u32, 4, target, output);
}
/// Helper function to convert f32 to a [u8; 4].
pub fn f32_to_u8(target: f32, output: &mut [u8]) {
    transmute_to_u8_from!(f32, 4, target, output); 
}

/// Function to bytewise copy from &[u8] to an &mut [u8], inserting at offset/
pub fn append_bytes(bytes: &[u8], buf: &mut [u8], offset: usize) -> usize {
    if bytes.len() + offset <= buf.len() {
        for i in 0 .. bytes.len() {
            buf[i + offset] = bytes[i];
        }
    }
    bytes.len()
}

/// Utility fn to zero out a given &mut [u8].
pub fn zero_u8_array(targ: &mut [u8]) {
    for i in 0 .. targ.len() {
        targ[i] = 0;
    }
}