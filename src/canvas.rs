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
    pub fn new(token: String) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(120)))
            .timeout_per_call(Some(Duration::from_secs(60)))
            .timeout_recv_response(Some(Duration::from_secs(30)))
            .timeout_recv_body(Some(Duration::from_secs(60)))
            .build()
            .into();
        Self { agent, token }
    }

    fn get_with_retry(&self, url: &str, with_auth: bool) -> Result<ureq::Body> {
        const MAX_ATTEMPTS: u32 = 5;
        let mut attempt: u32 = 0;
        loop {
            let mut req = self.agent.get(url);
            if with_auth {
                req = req.header("Authorization", &format!("Bearer {}", self.token));
            }
            match req.call() {
                Ok(r) => return Ok(r.into_body()),
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
        Ok(self.get_with_retry(url, true)?.read_to_string()?)
    }

    fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        Ok(serde_json::from_str(&self.get_string(url)?)?)
    }

    pub fn list_master_modules(&self) -> Result<Vec<Module>> {
        self.get_json(&format!(
            "{BASE}/courses/{SYLLABUS_REGISTRY_COURSE}/modules?include[]=items&per_page=100"
        ))
    }

    pub fn list_subcourse_modules(&self, sis_id: &str) -> Result<Vec<Module>> {
        self.get_json(&format!(
            "{BASE}/courses/sis_course_id:{sis_id}/modules?include[]=items&per_page=20"
        ))
    }

    pub fn get_file_meta(&self, api_url: &str) -> Result<FileMeta> {
        self.get_json(api_url)
    }


    pub fn download_bytes(&self, url: &str) -> Result<Vec<u8>> {
        Ok(self.get_with_retry(url, false)?.read_to_vec()?)
    }
}
