"""Import the public CourseWeave snapshot; no authentication or student data."""
import hashlib
import json
import re


def merge_snapshot(path, courses, sections):
    raw = path.read_bytes()
    snapshot = json.loads(raw)
    meta = snapshot['term']
    term = meta['id']
    assert term == 'S27' and meta['year'] == 2027 and meta['season'] == 'spring'
    assert snapshot['courses'], 'Empty snapshot'
    seen = set()
    count = 0
    def time(value):
        if value is None:
            return ''
        assert isinstance(value, int) and 0 <= value < 1440
        return f'{value // 60:02}:{value % 60:02}'
    for c in snapshot['courses']:
        code = c['code']
        assert re.fullmatch(r'\d{2}-\d{3}', code) and code not in seen
        seen.add(code)
        old = courses.get(code, {})
        courses[code] = {
            **old, 'code': code, 'name': c['title'], 'units': c['units'] or 0,
            'description': old.get('description', ''),
            'instructors': c['instructors'],
            'department': old.get('department', c['dept']),
            'level': str(int(code[-3:]) // 100 * 100),
            'offered_terms': sorted(set(old.get('offered_terms', []) + [term])),
            'gened_tags': old.get('gened_tags', []),
            'prerequisites': old.get('prerequisites', ''),
            'corequisites': old.get('corequisites', []),
            'antirequisites': old.get('antirequisites', []),
        }
        converted = {}
        for s in c['sections']:
            sid = f"courseweave:{term}:{code}:{s['kind']}:{s['name']}"
            assert sid not in converted, f'Duplicate section {sid}'
            meetings = []
            for m in s['meetings']:
                assert all(isinstance(d, int) and 0 <= d <= 6 for d in m['days'])
                meetings.append({'days': ['MTWRFSU'[d] for d in m['days']],
                                 'starts_at': time(m['start']), 'ends_at': time(m['end']),
                                 'location': ' '.join(str(m[k]) for k in ('building', 'room') if m.get(k))})
            converted[sid] = {'id': sid, 'label': s['name'], 'kind': s['kind'],
                              'instructors': s['instructors'], 'meetings': meetings,
                              'capacity': s.get('capacity'), 'parent': s.get('parent'),
                              'mini': s.get('mini'), 'mode': s.get('mode')}
        sections[code, term] = converted
        count += len(converted)
    return {'source': 'https://courseweave.jay3332.tech/data/S27.json',
            'sha256': hashlib.sha256(raw).hexdigest(), 'generated_at': snapshot['generatedAt'],
            'upstream_sources': meta.get('sources', []), 'term': term,
            'courses': len(seen), 'sections': count,
            'note': 'Third-party snapshot; generation time is not independently verified upstream freshness.'}
