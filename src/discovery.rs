use anyhow::Result;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

const MAX_LYEAR: i32 = 3;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Sem {
    Fall,
    Spring,
    Summer,
}

impl Sem {
    fn from_name(s: &str) -> Option<Self> {
        match s {
            "Fall" => Some(Sem::Fall),
            "Spring" => Some(Sem::Spring),
            "Summer" => Some(Sem::Summer),
            _ => None,
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(Sem::Fall),
            2 => Some(Sem::Spring),
            3 => Some(Sem::Summer),
            _ => None,
        }
    }

    fn id(self) -> u8 {
        match self {
            Sem::Fall => 1,
            Sem::Spring => 2,
            Sem::Summer => 3,
        }
    }

    pub fn ay_start(self, year: u32) -> i32 {
        if self == Sem::Fall {
            year as i32
        } else {
            year as i32 - 1
        }
    }
}

#[derive(Debug, Deserialize)]
struct FceRow {
    #[serde(rename = "Year")]
    year: String,
    #[serde(rename = "Sem")]
    sem: String,
    #[serde(rename = "Num")]
    num: String,
}

#[derive(Debug)]
pub enum Task {
    Info(String),
    Sections {
        course: String,
        lyear: i32,
        sem_id: u8,
    },
}

#[derive(Debug)]
pub struct Soc {
    term: (u32, Sem),
    pub codes: HashSet<String>,
}

fn dashed(num: &str) -> Option<String> {
    (num.len() == 5 && num.bytes().all(|b| b.is_ascii_digit()))
        .then(|| format!("{}-{}", &num[..2], &num[2..]))
}

fn parse_soc(text: &str) -> Option<Soc> {
    let term = text.lines().take(10).find_map(|l| {
        let rest = l.trim_start().strip_prefix("Semester:")?;
        let mut words = rest.split_whitespace();
        let sem = Sem::from_name(words.next()?)?;
        let year: u32 = words.find_map(|w| w.parse().ok())?;
        Some((year, sem))
    })?;
    let codes = text
        .lines()
        .filter_map(|l| l.strip_prefix('\t')?.split_once('\t').map(|(n, _)| n))
        .filter_map(dashed)
        .collect();
    Some(Soc { term, codes })
}

pub fn fetch_soc(season: &str) -> Result<Option<Soc>> {
    let url = format!("https://enr-apps.as.cmu.edu/assets/SOC/sched_layout_{season}.dat");
    let body = ureq::get(&url).call()?.into_body().read_to_string()?;
    Ok(parse_soc(&body))
}

pub fn parse_fce(path: &Path) -> Result<HashMap<String, HashSet<(u32, Sem)>>> {
    let mut out: HashMap<String, HashSet<(u32, Sem)>> = HashMap::new();
    for row in csv::Reader::from_path(path)?.deserialize::<FceRow>() {
        let row = row?;
        let Some(code) = dashed(&row.num) else {
            continue;
        };
        let Some(sem) = Sem::from_name(&row.sem) else {
            continue;
        };
        let Ok(year) = row.year.parse() else { continue };
        out.entry(code).or_default().insert((year, sem));
    }
    Ok(out)
}

pub fn course_tasks(
    course: &str,
    fce: &HashMap<String, HashSet<(u32, Sem)>>,
    soc: &HashMap<&str, Soc>,
    anchor: i32,
) -> Vec<Task> {
    let mut tasks = vec![Task::Info(course.to_string())];

    let fce_tuples = fce.get(course);
    let plan_window = course.starts_with("98-") && fce_tuples.is_none();

    if plan_window {
        for lyear in 0..=MAX_LYEAR {
            for s in [Sem::Fall, Sem::Spring, Sem::Summer] {
                tasks.push(Task::Sections {
                    course: course.to_string(),
                    lyear,
                    sem_id: s.id(),
                });
            }
        }
    } else {
        let mut tuples: HashSet<(u32, Sem)> = HashSet::new();
        if let Some(t) = fce_tuples {
            tuples.extend(t.iter().copied());
        }
        tuples.extend(
            soc.values()
                .filter(|s| s.codes.contains(course))
                .map(|s| s.term),
        );
        for (y, s) in tuples {
            let lyear = s.ay_start(y) - anchor + 1;
            if (0..=MAX_LYEAR).contains(&lyear) {
                tasks.push(Task::Sections {
                    course: course.to_string(),
                    lyear,
                    sem_id: s.id(),
                });
            }
        }
    }

    tasks
}
