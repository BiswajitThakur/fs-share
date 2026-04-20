use std::ffi::CStr;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use libc::ifaddrs;

pub struct IterIpAddr {
    head: *mut ifaddrs,
    curr: *mut ifaddrs,
}

unsafe impl Send for IterIpAddr {}

impl Drop for IterIpAddr {
    fn drop(&mut self) {
        if !self.head.is_null() {
            unsafe {
                libc::freeifaddrs(self.head);
            }
        }
    }
}

impl IterIpAddr {
    pub fn new() -> io::Result<Self> {
        let mut ifaddr: *mut ifaddrs = std::ptr::null_mut();
        unsafe {
            if libc::getifaddrs(&mut ifaddr) == -1 {
                Err(io::Error::last_os_error())
            } else {
                Ok(Self {
                    head: ifaddr,
                    curr: ifaddr,
                })
            }
        }
    }
    pub fn get_addr<U: AsRef<str>, T: AsRef<[U]>>(self, ifa_name: T) -> Option<IpAddr> {
        self.filter(|(i, _)| {
            ifa_name
                .as_ref()
                .iter()
                .find(|&a| a.as_ref() == i)
                .is_some()
        })
        .next()
        .map(|(_, v)| v)
    }
    pub fn iter_ipv4(self) -> impl Iterator<Item = (String, Ipv4Addr)> {
        self.filter_map(|(a, b)| match b {
            IpAddr::V4(addr) => Some((a, addr)),
            IpAddr::V6(_) => None,
        })
    }
    pub fn iter_ipv6(self) -> impl Iterator<Item = (String, Ipv6Addr)> {
        self.filter_map(|(a, b)| match b {
            IpAddr::V4(_) => None,
            IpAddr::V6(addr) => Some((a, addr)),
        })
    }
}

impl Iterator for IterIpAddr {
    type Item = (String, IpAddr);
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            return None;
        }
        unsafe {
            while !self.curr.is_null() {
                let ifa = &*self.curr;
                self.curr = ifa.ifa_next;

                if ifa.ifa_addr.is_null() {
                    continue;
                }

                let name = CStr::from_ptr(ifa.ifa_name).to_string_lossy().into_owned();
                let family = (*ifa.ifa_addr).sa_family as i32;

                match family {
                    libc::AF_INET => {
                        let sa = ifa.ifa_addr as *const libc::sockaddr_in;
                        let ip = Ipv4Addr::from(u32::from_be((*sa).sin_addr.s_addr));
                        return Some((name, IpAddr::V4(ip)));
                    }
                    libc::AF_INET6 => {
                        let sa6 = ifa.ifa_addr as *const libc::sockaddr_in6;
                        let ip = Ipv6Addr::from((*sa6).sin6_addr.s6_addr);
                        return Some((name, IpAddr::V6(ip)));
                    }
                    _ => continue,
                }
            }
        }
        None
    }
}
