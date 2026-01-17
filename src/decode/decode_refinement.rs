use crate::bitmap::Bitmap;
use crate::contexts::DecodingContext;
use crate::error::Jbig2Error;
const REFINEMENT_REUSED_CONTEXTS: [u16; 2] = [
    0x0100, // TPGRON start context for template 0
    0x0040, // TPGRON start context for template 1
];
/// Coding and reference templates for refinement decoding.
#[derive(Clone)]
pub struct RefinementTemplate {
    pub coding: Vec<(i8, i8)>,
    pub reference: Vec<(i8, i8)>,
}
/// Return the refinement template set for the given index.
pub fn get_refinement_template(index: usize) -> RefinementTemplate {
    match index {
        0 => RefinementTemplate {
            coding: vec![(-1, 0), (1, -1), (0, -1)],
            reference: vec![
                (1, 1),
                (0, 1),
                (-1, 1),
                (1, 0),
                (0, 0),
                (-1, 0),
                (1, -1),
                (0, -1),
            ],
        },
        1 => RefinementTemplate {
            coding: vec![(-1, 0), (1, -1), (0, -1), (-1, -1)],
            reference: vec![(1, 1), (0, 1), (1, 0), (0, 0), (-1, 0), (0, -1)],
        },
        _ => RefinementTemplate {
            coding: vec![],
            reference: vec![],
        },
    }
}
/// Inputs required to decode a refinement region.
#[derive(Clone)]
pub struct RefinementParams<'a> {
    pub width: usize,
    pub height: usize,
    pub template_index: usize,
    pub reference_bitmap: &'a Bitmap,
    pub offset_x: i32,
    pub offset_y: i32,
    pub prediction: bool,
    pub at: Vec<(i8, i8)>,
}
/// Decode a refinement bitmap using the reference bitmap and context templates.
pub fn decode_refinement<'a>(
    params: &RefinementParams<'a>,
    decoding_context: &mut DecodingContext,
) -> Result<Bitmap, Jbig2Error> {
    // Validate template index.
    if params.template_index > 1 {
        return Err(Jbig2Error::new("invalid refinement template index"));
    }
    // Validate AT parameters.
    if params.template_index == 0 && params.at.len() < 2 {
        return Err(Jbig2Error::new("template 0 requires 2 AT parameters"));
    }

    let mut coding_template = get_refinement_template(params.template_index).coding;
    if params.template_index == 0 {
        coding_template.push(params.at[0]);
    }
    let coding_template_length = coding_template.len();
    let coding_template_x = coding_template
        .iter()
        .map(|&(x, _)| x as i32)
        .collect::<Vec<_>>();
    let coding_template_y = coding_template
        .iter()
        .map(|&(_, y)| y as i32)
        .collect::<Vec<_>>();
    let mut reference_template = get_refinement_template(params.template_index).reference;
    if params.template_index == 0 {
        reference_template.push(params.at[1]);
    }
    let reference_template_length = reference_template.len();
    let reference_template_x = reference_template
        .iter()
        .map(|&(x, _)| x as i32)
        .collect::<Vec<_>>();
    let reference_template_y = reference_template
        .iter()
        .map(|&(_, y)| y as i32)
        .collect::<Vec<_>>();
    let reference_width = params.reference_bitmap.width;
    let reference_height = params.reference_bitmap.height;
    let reference_width_i32 = reference_width as i32;
    let reference_height_i32 = reference_height as i32;
    let width_i32 = params.width as i32;
    let start_context = REFINEMENT_REUSED_CONTEXTS[params.template_index];
    let mut contexts = decoding_context.get_contexts("GR");
    let mut decoder = decoding_context.get_decoder();
    let mut bitmap = Bitmap::new(params.width, params.height);
    let mut ltp = 0i32;
    for i in 0..params.height {
        if params.prediction {
            let sltp = decoder.read_bit(contexts.as_mut(), start_context as usize)? as i32;
            ltp ^= sltp;
        }
        for j in 0..params.width {
            let mut context_label = 0u16;
            let mut implicit = None;
            if params.prediction && ltp != 0 {
                let i_ref = j as i32 - params.offset_x;
                let j_ref = i as i32 - params.offset_y;
                let get_ref = |x: i32, y: i32| -> u8 {
                    if x < 0 || y < 0 || x >= reference_width_i32 || y >= reference_height_i32 {
                        0
                    } else {
                        params
                            .reference_bitmap
                            .get_pixel_unchecked(x as usize, y as usize)
                    }
                };
                let m = get_ref(i_ref, j_ref);
                if get_ref(i_ref - 1, j_ref - 1) == m
                    && get_ref(i_ref, j_ref - 1) == m
                    && get_ref(i_ref + 1, j_ref - 1) == m
                    && get_ref(i_ref - 1, j_ref) == m
                    && get_ref(i_ref + 1, j_ref) == m
                    && get_ref(i_ref - 1, j_ref + 1) == m
                    && get_ref(i_ref, j_ref + 1) == m
                    && get_ref(i_ref + 1, j_ref + 1) == m
                {
                    implicit = Some(m);
                }
            }
            if let Some(pixel) = implicit {
                bitmap.set_pixel_unchecked(j, i, pixel);
                continue;
            }
            for k in 0..coding_template_length {
                let i0 = i as i32 + coding_template_y[k];
                let j0 = j as i32 + coding_template_x[k];
                if i0 >= 0 && j0 >= 0 && j0 < width_i32 {
                    let bit = bitmap.get_pixel_unchecked(j0 as usize, i0 as usize) as u16;
                    if bit != 0 {
                        context_label |= 1 << k;
                    }
                }
            }
            for k in 0..reference_template_length {
                let i0 = i as i32 + reference_template_y[k] - params.offset_y;
                let j0 = j as i32 + reference_template_x[k] - params.offset_x;
                if i0 >= 0
                    && i0 < reference_height_i32
                    && j0 >= 0
                    && j0 < reference_width_i32
                {
                    let bit = params
                        .reference_bitmap
                        .get_pixel_unchecked(j0 as usize, i0 as usize) as u16;
                    if bit != 0 {
                        context_label |= 1 << (coding_template_length + k);
                    }
                }
            }
            let pixel = decoder.read_bit(contexts.as_mut(), context_label as usize)?;
            bitmap.set_pixel_unchecked(j, i, pixel);
        }
    }
    Ok(bitmap)
}
