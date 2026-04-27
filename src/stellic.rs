use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tracing::warn;

const BASE: &str = "https://academicaudit.andrew.cmu.edu";

#[derive(Debug, Deserialize)]
pub struct TermJoined {
    pub semester: u8,
    pub year: u32,
}

#[derive(Debug, Deserialize)]
struct Profile {
    username: String,
    default_plan_id: String,
    term_joined: TermJoined,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Program {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub program_type: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditVersion {
    pub id: u32,
    pub requirement: u64,
    pub name: String,
}

#[derive(Deserialize)]
struct ProgramsResp {
    programs: Vec<Program>,
}

#[derive(Deserialize)]
struct AuditVersionsResp {
    audits: Vec<AuditVersion>,
}

pub struct Stellic {
    agent: ureq::Agent,
    cookie: String,
    csrf: String,
    username: String,
    pub plan_id: String,
    out_dir: PathBuf,
}

impl Stellic {
    pub fn login(
        cookie: Option<String>,
        out_dir: PathBuf,
    ) -> Result<(Self, TermJoined)> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .timeout_per_call(Some(Duration::from_secs(60)))
            .timeout_recv_response(Some(Duration::from_secs(30)))
            .timeout_recv_body(Some(Duration::from_secs(45)))
            .build()
            .into();
        let mut cookie = cookie;
        loop {
            let c = match cookie.take() {
                Some(c) => c,
                None => obtain_cookie()?,
            };
            let Some(csrf) = c
                .split(';')
                .find_map(|p| p.trim().strip_prefix("csrftoken="))
                .map(str::to_string)
            else {
                warn!("cookie missing csrftoken; re-prompting");
                continue;
            };
            let body = agent
                .post(&format!("{BASE}/planner/getstudentprofile/"))
                .header("Cookie", &c)
                .header("X-CSRFToken", &csrf)
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Origin", BASE)
                .header("Referer", &format!("{BASE}/app/home"))
                .send("")?
                .into_body()
                .read_to_string()?;
            if !body.starts_with(")]}'") {
                warn!("auth response was not JSON (cookie expired or incomplete); re-prompting");
                continue;
            }
            let Profile {
                username,
                default_plan_id,
                term_joined,
            } = serde_json::from_str(strip_xssi(&body))?;
            return Ok((
                Self {
                    agent,
                    cookie: c,
                    csrf,
                    username,
                    plan_id: default_plan_id,
                    out_dir,
                },
                term_joined,
            ));
        }
    }

    fn get(&self, url: &str) -> Result<String> {
        const MAX_ATTEMPTS: u32 = 5;
        let mut attempt: u32 = 0;
        loop {
            let result = self
                .agent
                .get(url)
                .header("Cookie", &self.cookie)
                .header("X-CSRFToken", &self.csrf)
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Referer", &format!("{BASE}/app/courses"))
                .call();
            match result {
                Ok(r) => return Ok(r.into_body().read_to_string()?),
                Err(e) => {
                    if matches!(&e, ureq::Error::StatusCode(c) if (400..500).contains(c)) {
                        return Err(e.into());
                    }
                    attempt += 1;
                    if attempt >= MAX_ATTEMPTS {
                        return Err(e.into());
                    }
                    let delay_ms = (200u64 << attempt.min(5)).min(5000);
                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
            }
        }
    }

    fn post_json(&self, url: &str, body: &str) -> Result<String> {
        const MAX_ATTEMPTS: u32 = 5;
        let mut attempt: u32 = 0;
        loop {
            let result = self
                .agent
                .post(url)
                .header("Cookie", &self.cookie)
                .header("X-CSRFToken", &self.csrf)
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Content-Type", "application/json")
                .header("Origin", BASE)
                .header("Referer", &format!("{BASE}/app/home"))
                .send(body);
            match result {
                Ok(r) => return Ok(r.into_body().read_to_string()?),
                Err(e) => {
                    if matches!(&e, ureq::Error::StatusCode(c) if (400..500).contains(c)) {
                        return Err(e.into());
                    }
                    attempt += 1;
                    if attempt >= MAX_ATTEMPTS {
                        return Err(e.into());
                    }
                    let delay_ms = (200u64 << attempt.min(5)).min(5000);
                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
            }
        }
    }

    pub fn get_programs(&self) -> Result<Vec<Program>> {
        let body = self.get(&format!("{BASE}/catalog/getprograms/?campus_id=1"))?;
        let resp: ProgramsResp = serde_json::from_str(strip_xssi(&body))?;
        Ok(resp.programs)
    }

    pub fn get_audit_versions(&self, program_id: u32) -> Result<Vec<AuditVersion>> {
        let body = self.get(&format!(
            "{BASE}/planner/getauditversions/?program={program_id}&status=published"
        ))?;
        let resp: AuditVersionsResp = serde_json::from_str(strip_xssi(&body))?;
        Ok(resp.audits)
    }

    pub fn get_audit_data(&self, audit_id: u32) -> Result<serde_json::Value> {
        let body_obj = serde_json::json!({
            "student_username": self.username,
            "audit": audit_id,
            "default_audit_version": {"id": audit_id},
            "official": true,
        });
        let body = self.post_json(
            &format!("{BASE}/planner/getauditinfo/"),
            &body_obj.to_string(),
        )?;
        Ok(serde_json::from_str(strip_xssi(&body))?)
    }

    fn write_course(&self, course: &str, file: &str, contents: &str) -> Result<()> {
        let dir = self.out_dir.join(course.replace('-', ""));
        fs::create_dir_all(&dir)?;
        fs::write(dir.join(file), contents)?;
        Ok(())
    }

    pub fn save_info(&self, course: &str) -> Result<()> {
        let body = self.get(&format!(
            "{BASE}/catalog/getcourseinfo/?campus_id=1&course_code={course}&physical_year=2026"
        ))?;
        let mut json: serde_json::Value = serde_json::from_str(strip_xssi(&body))?;
        if json.get("success").and_then(|s| s.as_bool()) == Some(false) {
            return Ok(());
        }
        if let Some(obj) = json.as_object_mut() {
            // This is personal information
            obj.remove("student_context");
            obj.remove("enrollment_action_windows");
            obj.remove("alerts");
        }
        self.write_course(course, "info.json", &serde_json::to_string(&json)?)
    }

    pub fn save_sections(&self, course: &str, lyear: i32, sem_id: u8) -> Result<()> {
        let body = self.get(&format!(
            "{BASE}/planner/getcoursesections/?campus_id=1&course_code={course}&physical_year=2026&plan_id={}&sem_id={sem_id}&year={lyear}",
            self.plan_id
        ))?;
        let mut json: serde_json::Value = serde_json::from_str(strip_xssi(&body))?;
        let nonempty = json
            .get("data_list")
            .and_then(|d| d.as_array())
            .is_some_and(|a| !a.is_empty());
        if !nonempty {
            return Ok(());
        }
        if let Some(arr) = json.get_mut("data_list").and_then(|d| d.as_array_mut()) {
            for entry in arr {
                if let Some(obj) = entry.as_object_mut() {
                    obj.remove("current");
                }
            }
        }
        self.write_course(
            course,
            &format!("ly{lyear}_sm{sem_id}.json"),
            &serde_json::to_string(&json)?,
        )
    }
}

fn strip_xssi(body: &str) -> &str {
    body.split_once('\n').map(|(_, r)| r).unwrap_or(body)
}

fn obtain_cookie() -> Result<String> {
    eprintln!(
        "\nAuthenticate at https://academicaudit.andrew.cmu.edu, then in DevTools → Network,\n\
         click the request to https://academicaudit.andrew.cmu.edu/planner/getstudentprofile/,\n\
         copy the entire 'Cookie:' request header value, and paste it below:\n"
    );
    eprint!("> ");
    std::io::stderr().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let cookie = line.trim().to_string();
    if cookie.is_empty() {
        anyhow::bail!("empty cookie");
    }
    Ok(cookie)
}
