import collections
import json
from pathlib import Path
import tempfile
import unittest
from courseweave import merge_snapshot


class SnapshotTest(unittest.TestCase):
    def snapshot(self):
        return {'term': {'id': 'S27', 'year': 2027, 'season': 'spring'},
                'generatedAt': '2026-10-24T00:37:00Z', 'courses': [{
                    'code': '15-122', 'title': 'Imperative Computation', 'units': 12,
                    'dept': 'CS', 'instructors': ['Example Instructor'], 'sections': [{
                        'name': '1', 'kind': 'lecture', 'instructors': ['Example Instructor'],
                        'meetings': [{'days': [1, 3], 'start': 570, 'end': 650},
                                     {'days': [], 'start': None, 'end': None}]}]}]}

    def merge(self, data, courses=None):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'snapshot.json'
            path.write_text(json.dumps(data))
            courses = {} if courses is None else courses
            sections = collections.defaultdict(dict)
            report = merge_snapshot(path, courses, sections)
            return courses, sections, report

    def test_mapping_and_historical_metadata(self):
        courses, sections, report = self.merge(self.snapshot(), {
            '15-122': {'description': 'Retained description', 'offered_terms': ['F26']}})
        self.assertEqual(courses['15-122']['description'], 'Retained description')
        self.assertEqual(set(courses['15-122']['offered_terms']), {'S27', 'F26'})
        meetings = next(iter(sections['15-122', 'S27'].values()))['meetings']
        self.assertEqual(meetings[0]['days'], ['T', 'R'])
        self.assertEqual(meetings[0]['starts_at'], '09:30')
        self.assertEqual(meetings[1]['starts_at'], '')
        self.assertEqual(report['sections'], 1)

    def test_rejects_wrong_term_and_duplicate_sections(self):
        data = self.snapshot()
        data['term']['year'] = 2026
        with self.assertRaises(AssertionError): self.merge(data)
        data = self.snapshot()
        data['courses'][0]['sections'] *= 2
        with self.assertRaises(AssertionError): self.merge(data)


if __name__ == '__main__': unittest.main()
