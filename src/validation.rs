use crate::error::{
    ERR_DIMENSIONS_TOO_LARGE, ERR_INVALID_COMBINATION_OPERATOR, ERR_INVALID_DIMENSIONS,
    ERR_INVALID_REFERENCE_CORNER, ERR_INVALID_TEMPLATE_INDEX, ERR_TOO_MANY_SYMBOLS, Jbig2Error,
};

pub fn validate_bitmap_dimensions(width: usize, height: usize) -> Result<(), Jbig2Error> {
    if width == 0 || height == 0 {
        return Err(Jbig2Error::new(ERR_INVALID_DIMENSIONS));
    }
    if width > 65535 || height > 65535 {
        return Err(Jbig2Error::new(ERR_DIMENSIONS_TOO_LARGE));
    }
    Ok(())
}

pub fn validate_template_index(index: usize) -> Result<(), Jbig2Error> {
    if index > 3 {
        return Err(Jbig2Error::new(ERR_INVALID_TEMPLATE_INDEX));
    }
    Ok(())
}

pub fn validate_symbol_count(count: usize) -> Result<(), Jbig2Error> {
    if count > 65535 {
        return Err(Jbig2Error::new(ERR_TOO_MANY_SYMBOLS));
    }
    Ok(())
}

pub fn validate_reference_corner(corner: usize) -> Result<(), Jbig2Error> {
    if corner > 3 {
        return Err(Jbig2Error::new(ERR_INVALID_REFERENCE_CORNER));
    }
    Ok(())
}

pub fn validate_combination_operator(operator: usize) -> Result<(), Jbig2Error> {
    if operator > 7 {
        return Err(Jbig2Error::new(ERR_INVALID_COMBINATION_OPERATOR));
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
