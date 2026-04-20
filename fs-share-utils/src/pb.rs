pub trait ProgressBar {
    fn update(&self, size: u64);
    fn finish(&self);
}
