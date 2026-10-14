//! HTTP client for CMU's Canvas instance, scoped to what the syllabi pipeline
//! needs (module walks over the Syllabus Registry course, plus file metadata
//! fetches for downloadable attachments). Authentication is a user-generated
//! API token passed as a bearer header.

use anyhow::Result;
use serde::Deserialize;
use std::time::Duration;

const BASE: &str = "https://canvas.cmu.edu/api/v1";
pub const SYLLABUS_REGISTRY_COURSE: u32 = 3769;

#[derive(Debug, Deserialize, Clone)]
pub struct Module {
    pub name: String,
    #[serde(default)]
    pub items: Vec<ModuleItem>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModuleItem {
    pub title: String,
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub external_url: String,
}

#[derive(Debug, Deserialize)]
pub struct FileMeta {
    pub display_name: String,
    pub filename: String,
    pub url: String,
}

pub struct Canvas {
    agent: ureq::Agent,
    token: String,
}

impl Canvas {
    pub fn new(token: String, concurrency: usize) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(120)))
            .timeout_per_call(Some(Duration::from_secs(60)))
            .timeout_recv_response(Some(Duration::from_secs(30)))
            .timeout_recv_body(Some(Duration::from_secs(60)))
            .max_idle_connections(concurrency)
            .max_idle_connections_per_host(concurrency)
            .build()
            .into();
        Self { agent, token }
    }

    fn get_with_retry(
        &self,
        url: &str,
        with_auth: bool,
    ) -> Result<ureq::http::Response<ureq::Body>> {
        const MAX_ATTEMPTS: u32 = 5;
        let mut attempt: u32 = 0;
        loop {
            let mut req = self.agent.get(url);
            if with_auth {
                req = req.header("Authorization", &format!("Bearer {}", self.token));
            }

            match req.call() {
                Ok(r) => return Ok(r),
                Err(e) => {
                    if matches!(&e, ureq::Error::StatusCode(c) if (400..500).contains(c)) {
                        return Err(e.into());
                    }

                    attempt += 1;
                    if attempt >= MAX_ATTEMPTS {
                        return Err(e.into());
                    }

                    let delay = (200u64 << attempt.min(5)).min(5000);
                    std::thread::sleep(Duration::from_millis(delay));
                }
            }
        }
    }

    fn get_string(&self, url: &str) -> Result<String> {
        Ok(self
            .get_with_retry(url, true)?
            .into_body()
            .read_to_string()?)
    }

    fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        Ok(serde_json::from_str(&self.get_string(url)?)?)
    }

    fn get_all_pages<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<Vec<T>> {
        let mut out = Vec::new();
        let mut next = Some(url.to_string());
        while let Some(url) = next {
            let response = self.get_with_retry(&url, true)?;
            next = next_link(response.headers());

            let page: Vec<T> = serde_json::from_str(&response.into_body().read_to_string()?)?;
            out.extend(page);
        }

        Ok(out)
    }

    pub fn list_master_modules(&self) -> Result<Vec<Module>> {
        self.get_all_pages(&format!(
            "{BASE}/courses/{SYLLABUS_REGISTRY_COURSE}/modules?include[]=items&per_page=100"
        ))
    }

    pub fn list_subcourse_modules(&self, sis_id: &str) -> Result<Vec<Module>> {
        self.get_all_pages(&format!(
            "{BASE}/courses/sis_course_id:{sis_id}/modules?include[]=items&per_page=20"
        ))
    }

    pub fn get_file_meta(&self, api_url: &str) -> Result<FileMeta> {
        self.get_json(api_url)
    }

    pub fn download_bytes(&self, url: &str) -> Result<Vec<u8>> {
        Ok(self.get_with_retry(url, false)?.into_body().read_to_vec()?)
    }
}

fn next_link(headers: &ureq::http::HeaderMap) -> Option<String> {
    headers
        .get_all("link")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .find(|part| part.contains("rel=\"next\""))
        .and_then(|part| Some(part.split_once('<')?.1.split_once('>')?.0.to_string()))
}

#[cfg(test)]
mod tests {
    use super::next_link;
    use ureq::http::{HeaderMap, HeaderValue};

    #[test]
    fn next_link_picks_rel_next() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "link",
            HeaderValue::from_static(
                "<https://canvas.cmu.edu/api/v1/courses/3769/modules?page=1&per_page=100>; rel=\"current\",<https://canvas.cmu.edu/api/v1/courses/3769/modules?page=2&per_page=100>; rel=\"next\",<https://canvas.cmu.edu/api/v1/courses/3769/modules?page=1&per_page=100>; rel=\"first\"",
            ),
        );

        assert_eq!(
            next_link(&headers).as_deref(),
            Some("https://canvas.cmu.edu/api/v1/courses/3769/modules?page=2&per_page=100")
        );
    }

    #[test]
    fn next_link_absent_on_last_page() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "link",
            HeaderValue::from_static(
                "<https://canvas.cmu.edu/api/v1/courses/3769/modules?page=2&per_page=100>; rel=\"current\",<https://canvas.cmu.edu/api/v1/courses/3769/modules?page=1&per_page=100>; rel=\"first\"",
            ),
        );

        assert_eq!(next_link(&headers), None);
    }
}
