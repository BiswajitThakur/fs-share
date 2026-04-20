use fs_share_utils::pb::ProgressBar;

struct NoProgress;

impl ProgressBar for NoProgress {
    fn update(&self, _: u64) {}
    fn finish(&self) {}
}

struct MyPrograssBar {
    inner: indicatif::ProgressBar,
}

impl ProgressBar for MyPrograssBar {
    fn update(&self, size: u64) {
        self.inner.set_position(size);
    }
    fn finish(&self) {
        self.inner.finish();
    }
}

pub fn my_pb(n: u64) -> Box<dyn ProgressBar> {
    let pb = indicatif::ProgressBar::new(n);
    pb.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    Box::new(MyPrograssBar { inner: pb })
}

pub fn no_pb(_: u64) -> Box<dyn ProgressBar> {
    let pb = NoProgress;
    Box::new(pb)
}
