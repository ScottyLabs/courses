#!/usr/bin/env python3
"""Build independently sanitized public/member SQLite catalogs; inventory every input.
Raw exports stay outside the web root. No student context is copied to catalogs.
"""
from courseweave import merge_snapshot
import argparse, collections, csv, gzip, datetime as dt, hashlib, json, pathlib, re, sqlite3

def decode(x):
    if isinstance(x, dict):
        for k in ('$numberInt', '$numberDouble', '$numberLong'):
            if k in x: return float(x[k]) if k == '$numberDouble' else int(x[k])
        return {k: decode(v) for k,v in x.items()}
    if isinstance(x,list): return [decode(v) for v in x]
    return x

def records(p):
    text=p.read_text()
    try:
        x=json.loads(text)
        return decode(x if isinstance(x,list) else list(x.values()))
    except json.JSONDecodeError: return [decode(json.loads(l)) for l in text.splitlines() if l.strip()]

def term(year, season):
    s={1:'F',2:'S',3:'M','spring':'S','fall':'F','summer':'M'}.get(season)
    return f'{s}{int(year)%100:02}' if s else None

def number(v):
    try: return float(str(v or 0).split(",")[0])
    except ValueError: return 0

def clock(s):
    try: return dt.datetime.strptime(s.replace(' ',''),'%I:%M%p').strftime('%H:%M')
    except (ValueError,AttributeError): return ''

def main():
    ap=argparse.ArgumentParser();ap.add_argument('source',type=pathlib.Path);ap.add_argument('--output',type=pathlib.Path,default=pathlib.Path('data/catalog'));ap.add_argument('--stellic-root',type=pathlib.Path,help='Fresh per-user export directory; overrides historical info/sections');ap.add_argument('--courseweave',type=pathlib.Path,help='Public CourseWeave S27 JSON snapshot');a=ap.parse_args();a.output.mkdir(parents=True,exist_ok=True)
    inventory=[]; errors=[]
    cached = a.output/'inventory.json'
    prior = {r['path']:r for r in json.loads(cached.read_text())} if cached.exists() else {}
    for p in sorted(a.source.rglob('*')):
        if not p.is_file() or p.name=='.DS_Store': continue
        rel=str(p.relative_to(a.source));stat=p.stat();old=prior.get(rel)
        if old and old.get('mtime_ns')==stat.st_mtime_ns and old['bytes']==stat.st_size:inventory.append(old);continue
        b=p.read_bytes();inventory.append({'path':rel,'bytes':len(b),'mtime_ns':stat.st_mtime_ns,'sha256':hashlib.sha256(b).hexdigest()})
    cached.write_text(json.dumps(inventory,indent=2))
    history_roots=[a.source/'backup/exported/courses_history']
    if a.stellic_root:
        fresh=a.stellic_root/'courses_history'
        if not fresh.is_dir():raise ValueError('Missing fresh courses_history directory')
        history_roots.append(fresh)
    courses={};sections=collections.defaultdict(dict);instructors={};geneds={};fces=collections.defaultdict(list);programs=[];dates=[];offering_years=[]
    for c in records(a.source/'courses-files/courses.json'):
        code=c['courseID'];courses[code]={'code':code,'name':c['name'],'description':c.get('desc',''),'units':number(c.get('units')),'department':c.get('department',code[:2]),'level':str(int(code[-3:])//100*100) if code[-3:].isdigit() else '0','offered_terms':[],'gened_tags':[],'prerequisites':c.get('prereqString',''),'corequisites':c.get('coreqs',[]),'antirequisites':[]}
    def instr(i,code):
        ident=str(i['id']);name=' '.join(filter(None,[i.get('first_name'),i.get('last_name')]))
        row=instructors.setdefault(ident,{'id':ident,'name':name,'courses':[]})
        if code not in row['courses']:row['courses'].append(code)
        return name
    for p in (p for root in history_roots for p in sorted(root.glob('*/info.json'))):
        c=json.loads(p.read_text());code=c.get('code')
        if not code:errors.append(str(p));continue
        old=courses.get(code,{})
        ts=sorted({term(s['year'],s['semester']) for o in c.get('offerings',[]) for s in o.get('semesters',[]) }-{None})
        offering_years.extend(s['year'] for o in c.get('offerings',[]) for s in o.get('semesters',[]))
        courses[code]={'code':code,'name':c.get('name') or old.get('name',''),'description':c.get('long_desc') or old.get('description',''),'units':c.get('units') or 0,'department':old.get('department',code[:2]),'level':str(int(code[-3:])//100*100) if code[-3:].isdigit() else '0','offered_terms':ts,'gened_tags':old.get('gened_tags',[]),'prerequisites':(c.get('prereqs') or {}).get('text',''),'corequisites':[x['code'] for x in c.get('co_reqs',[]) if x.get('code')],'antirequisites':[x['code'] for x in c.get('anti_reqs',[]) if x.get('code')],'instructors':[instr(i,code) for i in c.get('instructors',[])]}
    # Legacy schedules supply explicit year/season, then Stellic sections supersede matching terms.
    for c in records(a.source/'courses-files/schedules.json'):
        code=c['courseID'];t=term(c['year'],c['semester'])
        if code not in courses or not t:continue
        for kind in ['lectures','sections']:
            for s in c.get(kind,[]):
                sid=f"legacy:{t}:{code}:{s['name']}";meetings=[]
                for m in (s.get('times') or []):
                    meetings.append({'days':[['U','M','T','W','R','F','S'][int(d)] for d in (m.get('days') or []) if 0<=int(d)<=6],'starts_at':clock(m.get('begin')),'ends_at':clock(m.get('end')),'location':' '.join(str(x) for x in [m.get('building'),m.get('room')] if x)})
                sections[code,t][sid]={'id':sid,'label':s['name'],'kind':kind,'instructors':s.get('instructors',[]),'meetings':meetings}
    days=dict(zip(['monday','tuesday','wednesday','thursday','friday','saturday','sunday'],'MTWRFSU'))
    replaced=set()
    for p in (p for root in history_roots for p in sorted(root.glob('*/*sm*.json'))):
        x=json.loads(p.read_text())
        if x.get('success') is False:errors.append(str(p));continue
        for c in x.get('data_list',[]):
            code=c['course']['code']
            if code not in courses:continue
            for g in c.get('section_groups',[]):
                for s in g.get('sections',[]):
                    date=s.get('start_date')
                    if not date:errors.append(str(p)+': undated section');continue
                    dates.append(date);year,month=map(int,date.split('-')[:2]);t=term(year,2 if month<=4 else 3 if month<=7 else 1)
                    if (str(p.parent.parent),code,t) not in replaced:sections[code,t]={};replaced.add((str(p.parent.parent),code,t))
                    meetings=[{'days':[days[d] for d in (m.get('days') or []) if d in days],'starts_at':clock(m.get('time',{}).get('start')),'ends_at':clock(m.get('time',{}).get('end')),'location':' '.join(str(v) for v in [m.get('building'),m.get('room')] if v)} for m in s.get('timings',[])]
                    sid=str(s['id']);sections[code,t][sid]={'id':sid,'label':s.get('name',''),'kind':g.get('group_name',''),'instructors':[instr(i,code) for i in s.get('instructors',[])],'meetings':meetings,'capacity':s.get('max_enrollment'),'waitlist':s.get('waitlist')}
    courseweave_path = a.courseweave or a.source/'courseweave/S27.json'
    courseweave = merge_snapshot(courseweave_path, courses, sections) if courseweave_path.exists() else None
    for g in records(a.source/'courses-files/geneds.json'):
        for tag in g.get('tags',[]):
            ident=g['school']+':'+tag;row=geneds.setdefault(ident,{'id':ident,'name':ident,'courses':[],'description':'Source updated '+g.get('lastUpdated','unknown')})
            code=g['courseID'];row['courses'].append(code)
            if code in courses:courses[code]['gened_tags'].append(ident)
    # The CSV is newer (includes 2026); Mongo FCE exports contain repeated history.
    fce_rows=[]
    with (a.source/'dalmatian_courses/fce_data.csv').open() as file:
        for r in csv.DictReader(file):
            code=re.sub(r'[^0-9]', '',r['Num'])[-5:]
            if len(code)!=5:continue
            code=code[:2]+'-'+code[2:]
            def rating(k):
                try:return float(r[k])
                except (ValueError,KeyError):return None
            f={'courseID':code,'year':r['Year'],'semester':r['Sem'],'college':r['College'],'numRespondents':number(r['# Responses']),'hrsPerWeek':rating('Hrs Per Week'),'rating':[rating('Overall teaching rate'),rating('Overall course rate')]}
            fce_rows.append(f)
            if f['college']!='Teaching Assistants':fces[code].append(f)
    for code,rows in fces.items():
        if code not in courses:continue
        summary={}
        for key,get in [('hours_per_week',lambda r:r.get('hrsPerWeek')),('overall_course',lambda r:(r.get('rating') or [None])[-1]),('overall_teaching',lambda r:r['rating'][-2] if len(r.get('rating',[]))>=2 else None)]:
            vals=[(get(r),r.get('numRespondents',0)) for r in rows];vals=[(v,n) for v,n in vals if isinstance(v,(float,int)) and n>0]
            if vals:summary[key]=sum(v*n for v,n in vals)/sum(n for v,n in vals)
        summary['semesters']=len({(r.get('year'),r.get('semester')) for r in rows});courses[code]['fce']=summary
    for syllabus in records(a.source/'dalmatian_courses/courseSyllabi.json'):
        raw=str(syllabus.get('number','')).replace('-','');code=raw[:2]+'-'+raw[2:]
        if code in courses and str(syllabus.get('url','')).startswith('https://canvas.cmu.edu/'):
            courses[code].setdefault('syllabi',[]).append({'label':f"{syllabus.get('season','')}{syllabus.get('year','')} {syllabus.get('section','')}",'url':syllabus['url']})
    for p in sorted((a.source/'backup/exported/programs').glob('*/*.json')):
        x=json.loads(p.read_text());programs.append({'id':f"{x['catalog_id']}:{x['audit_id']}",'name':x['program_name'],'audit_name':x['audit_name'],'program_type':x['program_type'],'tree':x['tree'],'source':'Stellic export','retrieved_at':None})
    for (code,t),ss in sections.items():
        if t not in courses[code]['offered_terms']:courses[code]['offered_terms'].append(t)
    public_programs=[]
    current=pathlib.Path('data/programs-current/manifest.json')
    if current.exists(): public_programs=json.loads(current.read_text())['programs']
    terms=sorted({t for c in courses.values() for t in c['offered_terms']},key=lambda t:(int(t[1:]),{'S':0,'M':1,'F':2}[t[0]]),reverse=True)
    # A fresh DB for each audience: restricted bytes never enter the public file, including free pages.
    for audience in ['public','member']:
        path=a.output/f'{audience}.sqlite';tmp=path.with_suffix('.tmp');tmp.unlink(missing_ok=True);db=sqlite3.connect(tmp)
        db.executescript('''CREATE TABLE catalog_metadata(schema_version INTEGER); INSERT INTO catalog_metadata VALUES(1);
CREATE TABLE catalog_coverage(dataset TEXT,term_id TEXT,PRIMARY KEY(dataset,term_id));
CREATE TABLE catalog_terms(id TEXT PRIMARY KEY,sort_order INTEGER,payload_json TEXT);
CREATE TABLE catalog_courses(term_id TEXT,code TEXT,name TEXT,description TEXT,department TEXT,level INTEGER,units REAL,offered_in TEXT,overall_course REAL,payload_json TEXT,PRIMARY KEY(term_id,code));
CREATE TABLE catalog_instructors(id TEXT PRIMARY KEY,name TEXT,department TEXT,payload_json TEXT);
CREATE TABLE catalog_geneds(id TEXT PRIMARY KEY,payload_json TEXT);
CREATE TABLE programs(id TEXT PRIMARY KEY,name TEXT,payload_json TEXT);
CREATE INDEX course_department ON catalog_courses(term_id,department);''')
        for dataset in ['terms','courses','geneds']+(['instructors'] if audience=='member' else []):db.execute('INSERT INTO catalog_coverage VALUES(?,?)',(dataset,''))
        for n,t in enumerate(terms):
            label={'F':'Fall','S':'Spring','M':'Summer'}[t[0]]+' 20'+t[1:];db.execute('INSERT INTO catalog_terms VALUES(?,?,?)',(t,n,json.dumps({'id':t,'label':label})));db.execute('INSERT INTO catalog_coverage VALUES(?,?)',('courses',t))
        for c in sorted(courses.values(),key=lambda c:c['code']):
            for t in ['']+sorted(set(c['offered_terms'])):
                row=dict(c);row['sections']=list(sections.get((c['code'],t),{}).values()) if t else []
                if audience=='public':
                    row.pop('fce',None);row.pop('instructors',None);row.pop('syllabi',None)
                    row['sections']=[{**s,'instructors':[],'meetings':[{k:v for k,v in m.items() if k!='location'} for m in s['meetings']]} for s in row['sections']]
                seasons='|'.join(sorted({{'F':'fall','S':'spring','M':'summer'}[v[0]] for v in c['offered_terms']}))
                db.execute('INSERT INTO catalog_courses VALUES(?,?,?,?,?,?,?,?,?,?)',(t,c['code'],c['name'],c['description'],c['department'],int(c['level']),c['units'],seasons,row.get('fce',{}).get('overall_course'),json.dumps(row)))
        if audience=='member':
            for i in instructors.values():db.execute('INSERT INTO catalog_instructors VALUES(?,?,?,?)',(i['id'],i['name'],i.get('department',''),json.dumps(i)))
            for p in programs:db.execute('INSERT INTO programs VALUES(?,?,?)',(p['id'],p['name'],json.dumps(p)))
        for p in public_programs:db.execute('INSERT INTO programs VALUES(?,?,?)',(p['id'],p['name'],json.dumps(p)))
        for g in geneds.values():db.execute('INSERT INTO catalog_geneds VALUES(?,?)',(g['id'],json.dumps(g)))
        db.commit();assert db.execute('PRAGMA integrity_check').fetchone()[0]=='ok';db.close();tmp.replace(path)
        browser_path=a.output/f'{audience}-browser.sqlite'
        browser_path.unlink(missing_ok=True)
        with sqlite3.connect(path) as source, sqlite3.connect(browser_path) as browser:
            source.backup(browser)
            keep=['']+terms[:3]
            marks=','.join('?' for _ in keep)
            browser.execute(f'DELETE FROM catalog_courses WHERE term_id NOT IN ({marks})',keep)
            browser.execute(f"DELETE FROM catalog_coverage WHERE dataset='courses' AND term_id NOT IN ({marks})",keep)
            browser.execute('DELETE FROM programs')
            browser.commit();browser.execute('VACUUM')
        with browser_path.open('rb') as source,gzip.open(str(browser_path)+'.gz','wb',compresslevel=6) as dest:dest.write(source.read())
    report={'courseweave':courseweave,'imported_at':dt.datetime.now(dt.timezone.utc).isoformat(),'source_files':len(inventory),'source_bytes':sum(x['bytes'] for x in inventory),'courses':len(courses),'section_records':sum(len(x) for x in sections.values()),'instructors':len(instructors),'gened_categories':len(geneds),'public_program_pages':len(public_programs),'program_audits':len(programs),'programs':len({p['id'].split(':')[0] for p in programs}),'section_dates':[min(dates),max(dates)],'offering_years':[min(offering_years),max(offering_years)],'terms':terms,'fce_rows':len(fce_rows),'fce_years':sorted({str(r.get('year')) for r in fce_rows}),'warnings':errors,'notes':['File modification times are copy dates, not verified scrape dates.','Stellic student_context/alerts and all unrecognized fields are excluded.','All original files are inventoried; not all datasets are exposed. Advanced credit, historical FCE rows and old snapshots remain in source; syllabi links are member-only.','Program trees require CMU authentication and are not a validated degree audit engine.','FCE summary is respondent-weighted across the supplied history.']}
    (a.output/'inventory.json').write_text(json.dumps(inventory,indent=2));(a.output/'report.json').write_text(json.dumps(report,indent=2));print(json.dumps({k:v for k,v in report.items() if k not in ['warnings','terms']},indent=2));print('warnings',len(errors))
if __name__=='__main__':main()
