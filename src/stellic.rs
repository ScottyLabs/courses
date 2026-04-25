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
    default_plan_id: String,
    term_joined: TermJoined,
}

pub struct Stellic {
    agent: ureq::Agent,
    cookie: String,
    csrf: String,
    pub plan_id: String,
    out_dir: PathBuf,
}

impl Stellic {
    pub fn login(
        cookie: Option<String>,
        andrew_id: &str,
        out_dir: PathBuf,
    ) -> Result<(Self, TermJoined)> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
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
                .send(&format!("student_username={andrew_id}"))?
                .into_body()
                .read_to_string()?;
            if !body.starts_with(")]}'") {
                warn!("auth response was not JSON (cookie expired or incomplete); re-prompting");
                continue;
            }
            let Profile {
                default_plan_id,
                term_joined,
            } = serde_json::from_str(strip_xssi(&body))?;
            return Ok((
                Self {
                    agent,
                    cookie: c,
                    csrf,
                    plan_id: default_plan_id,
                    out_dir,
                },
                term_joined,
            ));
        }
    }

    fn get(&self, url: &str) -> Result<String> {
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
                    let delay_ms = (200u64 << attempt.min(5)).min(5000);
                    std::thread::sleep(Duration::from_millis(delay_ms));
                    attempt += 1;
                }
            }
        }
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
        let stripped = strip_xssi(&body);
        let json: serde_json::Value = serde_json::from_str(stripped)?;
        if json.get("success").and_then(|s| s.as_bool()) == Some(false) {
            return Ok(());
        }
        self.write_course(course, "info.json", stripped)
    }

    pub fn save_sections(&self, course: &str, lyear: i32, sem_id: u8) -> Result<()> {
        let body = self.get(&format!(
            "{BASE}/planner/getcoursesections/?campus_id=1&course_code={course}&physical_year=2026&plan_id={}&sem_id={sem_id}&year={lyear}",
            self.plan_id
        ))?;
        let stripped = strip_xssi(&body);
        let json: serde_json::Value = serde_json::from_str(stripped)?;
        let nonempty = json
            .get("data_list")
            .and_then(|d| d.as_array())
            .is_some_and(|a| !a.is_empty());
        if !nonempty {
            return Ok(());
        }
        self.write_course(course, &format!("ly{lyear}_sm{sem_id}.json"), stripped)
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
