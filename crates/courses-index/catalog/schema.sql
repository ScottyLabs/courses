CREATE TABLE catalog_metadata(schema_version INTEGER); INSERT INTO catalog_metadata VALUES(1);
CREATE TABLE catalog_coverage(dataset TEXT,term_id TEXT,PRIMARY KEY(dataset,term_id));
CREATE TABLE catalog_terms(id TEXT PRIMARY KEY,sort_order INTEGER,payload_json TEXT);
CREATE TABLE catalog_courses(term_id TEXT,code TEXT,name TEXT,description TEXT,department TEXT,level INTEGER,units REAL,offered_in TEXT,overall_course REAL,payload_json TEXT,PRIMARY KEY(term_id,code));
CREATE TABLE catalog_instructors(id TEXT PRIMARY KEY,name TEXT,department TEXT,payload_json TEXT);
CREATE TABLE catalog_geneds(id TEXT PRIMARY KEY,payload_json TEXT);
CREATE TABLE programs(id TEXT PRIMARY KEY,name TEXT,payload_json TEXT);
CREATE INDEX course_department ON catalog_courses(term_id,department);
