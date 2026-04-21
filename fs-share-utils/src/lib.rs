//! # fs-share-utils
//!
//! Utility library for the `fs-share` project, providing reusable building blocks
//! for peer discovery, networking, file transfer, and progress tracking.
//!
//! This crate is designed to be lightweight, cross-platform, and suitable for
//! CLI-based file sharing over LAN.
//!
//!
//! ## Modules
//!
//! ### [`broadcast`]
//! Provides UDP broadcast utilities for peer discovery.
//! Used to announce and detect available senders/receivers on the network.
//!
//! ### [`ip`]
//! Utilities for working with network interfaces and IP addresses.
//! Includes platform-specific implementations (Linux, Windows, Android).
//!
//! ### [`pb`]
//! Progress bar utilities.
//! Abstracts progress reporting (can be enabled/disabled depending on CLI flags).
//!
//! ### [`receiver`]
//! Core logic for receiving files over TCP.
//! Handles incoming streams, parsing metadata, and saving files.
//!
//! ### [`sender`]
//! Core logic for sending files over TCP.
//! Responsible for encoding metadata and streaming file contents.
//!
//! ### [`tf`]
//! Transfer-related helpers (shared logic between sender and receiver).
//! Includes utilities for stream upgrade, protocol handling, and data framing.
//!
pub mod broadcast;
pub mod ip;
pub mod pb;
pub mod receiver;
pub mod sender;
pub mod tf;
