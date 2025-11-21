use crate::error::{Jbig2Error};

pub fn validate_bitmap_dimensions(width: usize, height: usize) -> Result<(), Jbig2Error> {
    // Zero dimensions are allowed (empty bitmaps)
    if width == 0 && height == 0 {
        return Ok(());
    }
    
    // Prevent integer overflow when calculating bitmap buffer size
    // stride = ((width - 1) / 8) + 1 bytes per row
    // total_size = stride * height must not exceed INT32_MAX
    let width_u32 = width as u32;
    let height_u32 = height as u32;
    let stride = if width_u32 == 0 { 1 } else { ((width_u32 - 1) / 8) + 1 };
    
    if height_u32 > (i32::MAX as u32) / stride {
        return Err(Jbig2Error::dimensions_too_large(width, height, i32::MAX as usize));
    }
    
    Ok(())
}

pub fn validate_template_index(index: usize) -> Result<(), Jbig2Error> {
    if index > 3 {
        return Err(Jbig2Error::invalid_template_index(index, 3));
    }
    Ok(())
}

pub fn validate_symbol_count(count: usize) -> Result<(), Jbig2Error> {
    const MAX_SYMBOLS: usize = 16777215; // 2^24 - 1, maximum for 3-byte field
    if count > MAX_SYMBOLS {
        return Err(Jbig2Error::too_many_symbols(count, MAX_SYMBOLS));
    }
    Ok(())
}

pub fn validate_reference_corner(corner: usize) -> Result<(), Jbig2Error> {
    if corner > 3 {
        return Err(Jbig2Error::invalid_reference_corner(corner as u8));
    }
    Ok(())
}

pub fn validate_combination_operator(operator: usize) -> Result<(), Jbig2Error> {
    if operator > 7 {
        return Err(Jbig2Error::invalid_combination_operator(operator as u8));
    }
    Ok(())
}

pub fn validate_generic_decode_params(
    width: usize,
    height: usize,
    template_index: usize,
) -> Result<(), Jbig2Error> {
    validate_bitmap_dimensions(width, height)?;
    validate_template_index(template_index)?;
    Ok(())
}

pub fn validate_text_decode_params(
    width: usize,
    height: usize,
    reference_corner: usize,
    combination_operator: usize,
) -> Result<(), Jbig2Error> {
    validate_bitmap_dimensions(width, height)?;
    validate_reference_corner(reference_corner)?;
    validate_combination_operator(combination_operator)?;
    Ok(())
}

pub fn validate_symbol_decode_params(
    template_index: usize,
    number_of_new_symbols: usize,
) -> Result<(), Jbig2Error> {
    validate_template_index(template_index)?;
    validate_symbol_count(number_of_new_symbols)?;
    Ok(())
}
