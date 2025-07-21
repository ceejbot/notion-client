use reqwest::Client;

pub mod complete;
pub mod create;
pub mod helpers;
pub mod list;
pub mod retrieve;
pub mod send;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct FileUploadsEndpoint {
    pub(super) client: Client,
}
