#[cfg(test)]
mod tests {
    use crate::arithmetic::ArithmeticDecoder;

    #[test]
    fn test_arithmetic_initialization_trace() {
        // Data from symbol_dictionary.jb2 trace
        // 0x94, 0x4F, 0x06, 0x7B
        let data = vec![0x94, 0x4F, 0x06, 0x7B];
        let mut decoder = ArithmeticDecoder::new(&data);
        
        println!("After new():");
        println!("  chigh: {:08x}", decoder.chigh);
        println!("  clow:  {:08x}", decoder.clow);
        println!("  ct:    {}", decoder.ct);
        println!("  a:     {:04x}", decoder.a);
        
        // Calculate effective C (high 32 bits)
        // In this implementation, C is split into chigh (high 16) and clow (low 16)
        // But chigh is u32 and clow is u32.
        // Let's see how they map.
        // If C = 0x3B000000
        // That's roughly chigh=0x3B00, clow=0x0000 ?
    }
}
