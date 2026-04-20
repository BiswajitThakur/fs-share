#[cfg(all(unix, not(target_os = "android")))]
mod unix;

#[cfg(all(unix, not(target_os = "android")))]
pub use unix::IterIpAddr;
