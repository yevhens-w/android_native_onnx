/// Configuration constants for ONNX inference

/// Standard ImageNet input dimensions
pub const IMAGE_WIDTH: u32 = 224;
pub const IMAGE_HEIGHT: u32 = 224;

/// ImageNet normalization constants (ImageNet dataset statistics)
pub const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
pub const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Classification thresholds and limits
pub const TOP_K_PREDICTIONS: usize = 5;
pub const MIN_CLASSIFICATION_CLASSES: usize = 1000;

/// Fallback ImageNet class labels (first 15 classes)
pub const FALLBACK_LABELS: &[&str] = &[
    "tench",
    "goldfish", 
    "great white shark",
    "tiger shark",
    "hammerhead",
    "electric ray",
    "stingray",
    "cock",
    "hen",
    "ostrich",
    "brambling",
    "goldfinch",
    "house finch",
    "junco",
    "indigo bunting",
];

