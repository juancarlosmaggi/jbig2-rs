use std::fmt;

// Legacy error strings kept for older call sites; prefer Jbig2Error constructors.
pub const ERR_INVALID_DIMENSIONS: &str =
    "invalid bitmap dimensions: width and height must be positive";
pub const ERR_DIMENSIONS_TOO_LARGE: &str = "bitmap dimensions too large";
pub const ERR_INVALID_TEMPLATE_INDEX: &str = "invalid template index";
pub const ERR_TOO_MANY_SYMBOLS: &str = "too many symbols";
pub const ERR_INVALID_REFERENCE_CORNER: &str = "invalid reference corner";
pub const ERR_INVALID_COMBINATION_OPERATOR: &str = "invalid combination operator";

/// Optional contextual details attached to an error instance.
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    pub position: Option<usize>,
    pub segment_number: Option<u32>,
    pub segment_type: Option<usize>,
}

/// Specific error categories produced during parsing, validation, and decoding.
#[derive(Debug, Clone)]
pub enum Jbig2ErrorKind {
    // Parsing errors.
    InsufficientData {
        required: usize,
        available: usize,
    },
    InvalidSegment {
        reason: String,
    },
    UnknownSegmentLength,
    InvalidFieldValue {
        field: String,
        value: String,
    },

    // Validation errors.
    InvalidDimensions {
        width: usize,
        height: usize,
    },
    DimensionsTooLarge {
        width: usize,
        height: usize,
        max: usize,
    },
    InvalidTemplateIndex {
        index: usize,
        max: usize,
    },
    InvalidCombinationOperator {
        operator: u8,
    },
    InvalidReferenceCorner {
        corner: u8,
    },

    // Decoding errors.
    MmrDecodingFailed {
        reason: String,
    },
    ArithmeticDecodingFailed {
        reason: String,
    },
    HuffmanDecodingFailed {
        reason: String,
    },
    InvalidRunLength {
        length: i32,
    },

    // Resource errors.
    TooManySymbols {
        count: usize,
        max: usize,
    },
    InfiniteLoopDetected {
        context: String,
    },
    BufferOverrun {
        position: usize,
        limit: usize,
    },
    ResourceLimitExceeded {
        resource: &'static str,
        limit: usize,
        actual: usize,
    },
    Cancelled,
    MissingResource {
        resource: String,
    },

    // Feature errors.
    UnsupportedFeature {
        feature: String,
    },

    // Generic fallback for gradual migration.
    Other {
        message: String,
    },
}

/// Stable numeric error codes for FFI and non-Rust callers.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jbig2ErrorCode {
    Other = 1,
    InsufficientData = 2,
    InvalidSegment = 3,
    UnknownSegmentLength = 4,
    InvalidFieldValue = 5,
    InvalidDimensions = 6,
    DimensionsTooLarge = 7,
    InvalidTemplateIndex = 8,
    InvalidCombinationOperator = 9,
    InvalidReferenceCorner = 10,
    MmrDecodingFailed = 11,
    ArithmeticDecodingFailed = 12,
    HuffmanDecodingFailed = 13,
    InvalidRunLength = 14,
    TooManySymbols = 15,
    InfiniteLoopDetected = 16,
    BufferOverrun = 17,
    ResourceLimitExceeded = 18,
    Cancelled = 19,
    MissingResource = 20,
    UnsupportedFeature = 21,
}

impl Jbig2ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Other => "other",
            Self::InsufficientData => "insufficient_data",
            Self::InvalidSegment => "invalid_segment",
            Self::UnknownSegmentLength => "unknown_segment_length",
            Self::InvalidFieldValue => "invalid_field_value",
            Self::InvalidDimensions => "invalid_dimensions",
            Self::DimensionsTooLarge => "dimensions_too_large",
            Self::InvalidTemplateIndex => "invalid_template_index",
            Self::InvalidCombinationOperator => "invalid_combination_operator",
            Self::InvalidReferenceCorner => "invalid_reference_corner",
            Self::MmrDecodingFailed => "mmr_decoding_failed",
            Self::ArithmeticDecodingFailed => "arithmetic_decoding_failed",
            Self::HuffmanDecodingFailed => "huffman_decoding_failed",
            Self::InvalidRunLength => "invalid_run_length",
            Self::TooManySymbols => "too_many_symbols",
            Self::InfiniteLoopDetected => "infinite_loop_detected",
            Self::BufferOverrun => "buffer_overrun",
            Self::ResourceLimitExceeded => "resource_limit_exceeded",
            Self::Cancelled => "cancelled",
            Self::MissingResource => "missing_resource",
            Self::UnsupportedFeature => "unsupported_feature",
        }
    }
}

/// Primary error type used throughout the decoding pipeline.
#[derive(Debug, Clone)]
pub struct Jbig2Error {
    pub kind: Jbig2ErrorKind,
    pub context: Option<ErrorContext>,
}

impl Jbig2Error {
    /// Legacy constructor for backward compatibility; prefer specific helpers.
    pub fn new(msg: &str) -> Self {
        Jbig2Error {
            kind: Jbig2ErrorKind::Other {
                message: msg.to_string(),
            },
            context: None,
        }
    }

    // Constructors for parsing errors.

    pub fn insufficient_data(required: usize, available: usize) -> Self {
        Self {
            kind: Jbig2ErrorKind::InsufficientData {
                required,
                available,
            },
            context: None,
        }
    }

    pub fn invalid_segment(reason: impl Into<String>) -> Self {
        Self {
            kind: Jbig2ErrorKind::InvalidSegment {
                reason: reason.into(),
            },
            context: None,
        }
    }

    pub fn unknown_segment_length() -> Self {
        Self {
            kind: Jbig2ErrorKind::UnknownSegmentLength,
            context: None,
        }
    }

    pub fn invalid_field_value(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            kind: Jbig2ErrorKind::InvalidFieldValue {
                field: field.into(),
                value: value.into(),
            },
            context: None,
        }
    }

    // Constructors for validation errors.

    pub fn invalid_dimensions(width: usize, height: usize) -> Self {
        Self {
            kind: Jbig2ErrorKind::InvalidDimensions { width, height },
            context: None,
        }
    }

    pub fn dimensions_too_large(width: usize, height: usize, max: usize) -> Self {
        Self {
            kind: Jbig2ErrorKind::DimensionsTooLarge { width, height, max },
            context: None,
        }
    }

    pub fn invalid_template_index(index: usize, max: usize) -> Self {
        Self {
            kind: Jbig2ErrorKind::InvalidTemplateIndex { index, max },
            context: None,
        }
    }

    pub fn invalid_combination_operator(operator: u8) -> Self {
        Self {
            kind: Jbig2ErrorKind::InvalidCombinationOperator { operator },
            context: None,
        }
    }

    pub fn invalid_reference_corner(corner: u8) -> Self {
        Self {
            kind: Jbig2ErrorKind::InvalidReferenceCorner { corner },
            context: None,
        }
    }

    // Constructors for decoding errors.

    pub fn mmr_decoding_failed(reason: impl Into<String>) -> Self {
        Self {
            kind: Jbig2ErrorKind::MmrDecodingFailed {
                reason: reason.into(),
            },
            context: None,
        }
    }

    pub fn arithmetic_decoding_failed(reason: impl Into<String>) -> Self {
        Self {
            kind: Jbig2ErrorKind::ArithmeticDecodingFailed {
                reason: reason.into(),
            },
            context: None,
        }
    }

    pub fn huffman_decoding_failed(reason: impl Into<String>) -> Self {
        Self {
            kind: Jbig2ErrorKind::HuffmanDecodingFailed {
                reason: reason.into(),
            },
            context: None,
        }
    }

    // Constructors for resource and bounds errors.

    pub fn too_many_symbols(count: usize, max: usize) -> Self {
        Self {
            kind: Jbig2ErrorKind::TooManySymbols { count, max },
            context: None,
        }
    }

    pub fn infinite_loop_detected(context: impl Into<String>) -> Self {
        Self {
            kind: Jbig2ErrorKind::InfiniteLoopDetected {
                context: context.into(),
            },
            context: None,
        }
    }

    pub fn buffer_overrun(position: usize, limit: usize) -> Self {
        Self {
            kind: Jbig2ErrorKind::BufferOverrun { position, limit },
            context: None,
        }
    }

    pub fn resource_limit_exceeded(resource: &'static str, limit: usize, actual: usize) -> Self {
        Self {
            kind: Jbig2ErrorKind::ResourceLimitExceeded {
                resource,
                limit,
                actual,
            },
            context: None,
        }
    }

    pub fn cancelled() -> Self {
        Self {
            kind: Jbig2ErrorKind::Cancelled,
            context: None,
        }
    }

    pub fn missing_resource(resource: impl Into<String>) -> Self {
        Self {
            kind: Jbig2ErrorKind::MissingResource {
                resource: resource.into(),
            },
            context: None,
        }
    }

    pub fn unsupported_feature(feature: impl Into<String>) -> Self {
        Self {
            kind: Jbig2ErrorKind::UnsupportedFeature {
                feature: feature.into(),
            },
            context: None,
        }
    }

    // Fluent helpers for attaching context.

    pub fn with_position(mut self, pos: usize) -> Self {
        self.context
            .get_or_insert_with(ErrorContext::default)
            .position = Some(pos);
        self
    }

    pub fn with_segment(mut self, number: u32, seg_type: usize) -> Self {
        let ctx = self.context.get_or_insert_with(ErrorContext::default);
        ctx.segment_number = Some(number);
        ctx.segment_type = Some(seg_type);
        self
    }

    pub fn with_segment_number(mut self, number: u32) -> Self {
        self.context
            .get_or_insert_with(ErrorContext::default)
            .segment_number = Some(number);
        self
    }

    pub fn code(&self) -> Jbig2ErrorCode {
        use Jbig2ErrorKind::*;
        match &self.kind {
            InsufficientData { .. } => Jbig2ErrorCode::InsufficientData,
            InvalidSegment { .. } => Jbig2ErrorCode::InvalidSegment,
            UnknownSegmentLength => Jbig2ErrorCode::UnknownSegmentLength,
            InvalidFieldValue { .. } => Jbig2ErrorCode::InvalidFieldValue,
            InvalidDimensions { .. } => Jbig2ErrorCode::InvalidDimensions,
            DimensionsTooLarge { .. } => Jbig2ErrorCode::DimensionsTooLarge,
            InvalidTemplateIndex { .. } => Jbig2ErrorCode::InvalidTemplateIndex,
            InvalidCombinationOperator { .. } => Jbig2ErrorCode::InvalidCombinationOperator,
            InvalidReferenceCorner { .. } => Jbig2ErrorCode::InvalidReferenceCorner,
            MmrDecodingFailed { .. } => Jbig2ErrorCode::MmrDecodingFailed,
            ArithmeticDecodingFailed { .. } => Jbig2ErrorCode::ArithmeticDecodingFailed,
            HuffmanDecodingFailed { .. } => Jbig2ErrorCode::HuffmanDecodingFailed,
            InvalidRunLength { .. } => Jbig2ErrorCode::InvalidRunLength,
            TooManySymbols { .. } => Jbig2ErrorCode::TooManySymbols,
            InfiniteLoopDetected { .. } => Jbig2ErrorCode::InfiniteLoopDetected,
            BufferOverrun { .. } => Jbig2ErrorCode::BufferOverrun,
            ResourceLimitExceeded { .. } => Jbig2ErrorCode::ResourceLimitExceeded,
            Cancelled => Jbig2ErrorCode::Cancelled,
            MissingResource { .. } => Jbig2ErrorCode::MissingResource,
            UnsupportedFeature { .. } => Jbig2ErrorCode::UnsupportedFeature,
            Other { .. } => Jbig2ErrorCode::Other,
        }
    }

    pub fn code_name(&self) -> &'static str {
        self.code().as_str()
    }
}

impl fmt::Display for Jbig2Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Jbig2ErrorKind::*;

        write!(f, "Jbig2Error: ")?;

        // Describe the primary error variant.
        match &self.kind {
            InsufficientData {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient data (required: {} bytes, available: {} bytes)",
                    required, available
                )?;
            }
            InvalidSegment { reason } => {
                write!(f, "Invalid segment: {}", reason)?;
            }
            UnknownSegmentLength => {
                write!(f, "Unknown segment length")?;
            }
            InvalidFieldValue { field, value } => {
                write!(f, "Invalid value for field '{}': {}", field, value)?;
            }
            InvalidDimensions { width, height } => {
                write!(f, "Invalid dimensions: {}x{}", width, height)?;
            }
            DimensionsTooLarge { width, height, max } => {
                write!(
                    f,
                    "Dimensions too large: {}x{} (max: {})",
                    width, height, max
                )?;
            }
            InvalidTemplateIndex { index, max } => {
                write!(f, "Invalid template index: {} (max: {})", index, max)?;
            }
            InvalidCombinationOperator { operator } => {
                write!(f, "Invalid combination operator: {}", operator)?;
            }
            InvalidReferenceCorner { corner } => {
                write!(f, "Invalid reference corner: {}", corner)?;
            }
            MmrDecodingFailed { reason } => {
                write!(f, "MMR decoding failed: {}", reason)?;
            }
            ArithmeticDecodingFailed { reason } => {
                write!(f, "Arithmetic decoding failed: {}", reason)?;
            }
            HuffmanDecodingFailed { reason } => {
                write!(f, "Huffman decoding failed: {}", reason)?;
            }
            InvalidRunLength { length } => {
                write!(f, "Invalid run length: {}", length)?;
            }
            TooManySymbols { count, max } => {
                write!(f, "Too many symbols: {} (max: {})", count, max)?;
            }
            InfiniteLoopDetected { context } => {
                write!(f, "Infinite loop detected in {}", context)?;
            }
            BufferOverrun { position, limit } => {
                write!(
                    f,
                    "Buffer overrun at position {} (limit: {})",
                    position, limit
                )?;
            }
            ResourceLimitExceeded {
                resource,
                limit,
                actual,
            } => {
                write!(
                    f,
                    "Resource limit exceeded for {}: {} > {}",
                    resource, actual, limit
                )?;
            }
            Cancelled => {
                write!(f, "Decode cancelled")?;
            }
            MissingResource { resource } => {
                write!(f, "Missing required resource: {}", resource)?;
            }
            UnsupportedFeature { feature } => {
                write!(f, "Unsupported feature: {}", feature)?;
            }
            Other { message } => {
                write!(f, "{}", message)?;
            }
        }

        // Append contextual fields when present.
        if let Some(ctx) = &self.context
            && (ctx.position.is_some()
                || ctx.segment_number.is_some()
                || ctx.segment_type.is_some())
        {
            write!(f, " [")?;
            let mut first = true;

            if let Some(pos) = ctx.position {
                write!(f, "position: 0x{:x}", pos)?;
                first = false;
            }

            if let Some(seg_num) = ctx.segment_number {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "segment: {}", seg_num)?;
                first = false;
            }

            if let Some(seg_type) = ctx.segment_type {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "type: {}", seg_type)?;
            }

            write!(f, "]")?;
        }

        Ok(())
    }
}

impl std::error::Error for Jbig2Error {}
