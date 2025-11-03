pub fn log2(x: u32) -> u32 {
    if x == 0 { 0 } else { (x as f64).log2() as u32 }
}
