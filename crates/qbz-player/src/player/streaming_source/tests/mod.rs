//! Unit tests for `streaming_source`, split by theme: buffer-size ladder
//! (`tests_config`) vs. read/write/seek/error behavior on the buffer
//! itself (`tests_buffer`).

mod tests_buffer;
mod tests_buffer_errors;
mod tests_config;
