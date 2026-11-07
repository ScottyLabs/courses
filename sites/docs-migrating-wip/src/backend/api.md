# Courses API
ALl endpoints require special API authenticaion becasue htye release sensitive data (FCEs/Profs/Locations)

GET /catalog/code/{course-code}
 - Return JSON output of course correcponding to that code
 - 200 OK on success
 - 404 Not found if course doesnt exist

GET /catalog/search/{n,query}
  - Reutrn a JSON of the top n results related to the query



JSON Output format

```
{
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
