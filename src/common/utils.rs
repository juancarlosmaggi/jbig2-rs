/// Return the integer log2 rounded up for values greater than 1.
pub fn log2(x: u32) -> u32 {
    if x <= 1 { 0 } else { (x - 1).ilog2() + 1 }
}
