# Courses API

ALl endpoints require special API authenticaion becasue htye release sensitive data (FCEs/Profs/Locations)

GET /catalog/code/{course-code}

- Return JSON output of course correcponding to that code
- 200 OK on success
- 404 Not found if course doesnt exist

GET /catalog/search/{n,query}

- Reutrn a JSON of the top n results related to the query

GET /catalog/exams/{code,semester}

- Semester should eba letter followed by the last two digits of the year (i.e. F25, S27, N25, M24)
- Course code shoudl be valid
- 200 OK if success
- 404 if course isnt found for this semester

JSON Output format

```
course: {
  number: int, 
  current-name: string, 
  description: string, 
  prequisites: [numbers], 
  unlocks: [numbers], 
  sections: {
      section_name: string
      capcity: int
      times: datetime
      instructor: string
    }
  semesters_offered: [string], 
  instructor: string,
  fce: {
    [{
      semester, instructor, hours, reviews
    }]
  }
}
```

TODO:

- Filters on requirements, professor, timings, departments, etc
- Adding exam endpoints
-
