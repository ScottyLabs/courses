//! Categorical filter index. Each axis interns its values to dense `u16`
//! ids at build time and stores `Vec<RoaringBitmap>` indexed by id, so
//! lookup is one `Vec` indexed access. A query-time `HashMap<String, u16>`
//! per string axis maps user input back to ids; integer axes use direct
//! `HashMap<T, u16>` for the same purpose.

use std::collections::HashMap;

use roaring::RoaringBitmap;

use crate::doc::Course;

pub type FacetValueId = u16;

pub struct StringAxis {
    pub bitmaps: Vec<RoaringBitmap>,
    pub values: Vec<String>,
    /// Indices into `bitmaps`/`values` ordered by descending bitmap size.
    /// Top-K facet counting walks this order so a small K threshold can
    /// terminate early once `value_bitmap.len() < threshold`.
    pub sorted_by_size: Vec<u32>,
    lookup: HashMap<String, FacetValueId>,
}

impl Default for StringAxis {
    fn default() -> Self {
        Self::new()
    }
}

impl StringAxis {
    pub fn new() -> Self {
        Self {
            bitmaps: Vec::new(),
            values: Vec::new(),
            sorted_by_size: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    fn finalize(&mut self) {
        self.sorted_by_size = (0..self.bitmaps.len() as u32).collect();
        self.sorted_by_size
            .sort_by_key(|&i| std::cmp::Reverse(self.bitmaps[i as usize].len()));
    }

    fn id_for(&mut self, value: &str) -> FacetValueId {
        if let Some(&id) = self.lookup.get(value) {
            return id;
        }
        let id = self.values.len() as FacetValueId;
        self.values.push(value.to_string());
        self.bitmaps.push(RoaringBitmap::new());
        self.lookup.insert(value.to_string(), id);
        id
    }

    pub fn lookup(&self, value: &str) -> Option<&RoaringBitmap> {
        let id = *self.lookup.get(value)?;
        self.bitmaps.get(id as usize)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub struct IntAxis<T: Copy + Eq + std::hash::Hash + Ord> {
    pub bitmaps: Vec<RoaringBitmap>,
    pub values: Vec<T>,
    /// Indices into `bitmaps`/`values` ordered by descending bitmap size.
    pub sorted_by_size: Vec<u32>,
    lookup: HashMap<T, FacetValueId>,
}

impl<T: Copy + Eq + std::hash::Hash + Ord> Default for IntAxis<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + Eq + std::hash::Hash + Ord> IntAxis<T> {
    pub fn new() -> Self {
        Self {
            bitmaps: Vec::new(),
            values: Vec::new(),
            sorted_by_size: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    fn finalize(&mut self) {
        self.sorted_by_size = (0..self.bitmaps.len() as u32).collect();
        self.sorted_by_size
            .sort_by_key(|&i| std::cmp::Reverse(self.bitmaps[i as usize].len()));
    }

    fn id_for(&mut self, value: T) -> FacetValueId {
        if let Some(&id) = self.lookup.get(&value) {
            return id;
        }
        let id = self.values.len() as FacetValueId;
        self.values.push(value);
        self.bitmaps.push(RoaringBitmap::new());
        self.lookup.insert(value, id);
        id
    }

    pub fn lookup(&self, value: T) -> Option<&RoaringBitmap> {
        let id = *self.lookup.get(&value)?;
        self.bitmaps.get(id as usize)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub struct FacetIndex {
    pub dept: StringAxis,
    pub school: StringAxis,
    pub level: StringAxis,
    pub attribute_tags: StringAxis,
    pub gened_tags: StringAxis,
    pub skills: StringAxis,
    pub campuses: IntAxis<u32>,
    pub semesters_offered: IntAxis<(u16, u8)>,
    pub program_id: IntAxis<u32>,
    pub program_type: IntAxis<u8>,
    pub has_syllabus_terms: StringAxis,
}

impl FacetIndex {
    pub fn build(courses: &[Course]) -> Self {
        let mut idx = FacetIndex {
            dept: StringAxis::new(),
            school: StringAxis::new(),
            level: StringAxis::new(),
            attribute_tags: StringAxis::new(),
            gened_tags: StringAxis::new(),
            skills: StringAxis::new(),
            campuses: IntAxis::new(),
            semesters_offered: IntAxis::new(),
            program_id: IntAxis::new(),
            program_type: IntAxis::new(),
            has_syllabus_terms: StringAxis::new(),
        };
        for c in courses {
            let id = c.id;
            insert_str(&mut idx.dept, &c.dept, id);
            if let Some(s) = &c.school {
                insert_str(&mut idx.school, s, id);
            }
            if let Some(l) = &c.level {
                insert_str(&mut idx.level, l, id);
            }
            for tag in &c.attribute_tags {
                insert_str(&mut idx.attribute_tags, tag, id);
            }
            for tag in &c.gened_tags {
                insert_str(&mut idx.gened_tags, &tag.name, id);
            }
            for sk in &c.skills {
                insert_str(&mut idx.skills, sk, id);
            }
            for &cm in &c.campuses {
                insert_int(&mut idx.campuses, cm, id);
            }
            for so in &c.semesters_offered {
                insert_int(&mut idx.semesters_offered, (so.year, so.sem as u8), id);
            }
            for pm in &c.programs {
                insert_int(&mut idx.program_id, pm.program_id, id);
                insert_int(&mut idx.program_type, pm.program_type, id);
            }
            for term in &c.has_syllabus_terms {
                insert_str(&mut idx.has_syllabus_terms, term, id);
            }
        }
        idx.dept.finalize();
        idx.school.finalize();
        idx.level.finalize();
        idx.attribute_tags.finalize();
        idx.gened_tags.finalize();
        idx.skills.finalize();
        idx.has_syllabus_terms.finalize();
        idx.campuses.finalize();
        idx.semesters_offered.finalize();
        idx.program_id.finalize();
        idx.program_type.finalize();
        idx
    }

    pub fn cardinality(&self) -> usize {
        self.dept.len()
            + self.school.len()
            + self.level.len()
            + self.attribute_tags.len()
            + self.gened_tags.len()
            + self.skills.len()
            + self.campuses.len()
            + self.semesters_offered.len()
            + self.program_id.len()
            + self.program_type.len()
            + self.has_syllabus_terms.len()
    }
}

fn insert_str(axis: &mut StringAxis, key: &str, id: u32) {
    if key.is_empty() {
        return;
    }
    let value_id = axis.id_for(key);
    axis.bitmaps[value_id as usize].insert(id);
}

fn insert_int<T: Copy + Eq + std::hash::Hash + Ord>(axis: &mut IntAxis<T>, key: T, id: u32) {
    let value_id = axis.id_for(key);
    axis.bitmaps[value_id as usize].insert(id);
}
