pub trait Detector {
    fn key(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceholderDetector {
    key: &'static str,
}

impl PlaceholderDetector {
    pub fn new(key: &'static str) -> Self {
        Self { key }
    }
}

impl Detector for PlaceholderDetector {
    fn key(&self) -> &'static str {
        self.key
    }
}

pub struct DetectorRegistry {
    detectors: Vec<Box<dyn Detector>>,
}

impl DetectorRegistry {
    pub fn bootstrap() -> Self {
        Self {
            detectors: vec![Box::new(PlaceholderDetector::new("os"))],
        }
    }

    pub fn keys(&self) -> Vec<&'static str> {
        self.detectors
            .iter()
            .map(|detector| detector.key())
            .collect()
    }
}
