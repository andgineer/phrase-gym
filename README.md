# Repository Coverage

[Full report](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

| Name                                       |    Stmts |     Miss |   Cover |   Missing |
|------------------------------------------- | -------: | -------: | ------: | --------: |
| src/phrase\_gym/\_\_about\_\_.py           |        1 |        0 |    100% |           |
| src/phrase\_gym/coach/bounded\_process.py  |       46 |        3 |     93% | 28, 79-80 |
| src/phrase\_gym/coach/main.py              |       51 |       27 |     47% |39-47, 51, 58-67, 71-82 |
| src/phrase\_gym/coach/speech.py            |       87 |       10 |     89% |51-52, 101, 117, 123, 138, 161-162, 168-169 |
| src/phrase\_gym/server/api/coach.py        |       25 |        3 |     88% | 23, 31-32 |
| src/phrase\_gym/server/api/imports.py      |       30 |        6 |     80% |29-30, 36-39 |
| src/phrase\_gym/server/api/library.py      |       27 |        6 |     78% |12, 20-22, 27, 33 |
| src/phrase\_gym/server/api/recordings.py   |       50 |        7 |     86% |23-24, 26, 28, 37, 49, 89 |
| src/phrase\_gym/server/api/review.py       |       21 |       11 |     48% |12-15, 19-30 |
| src/phrase\_gym/server/api/settings.py     |       15 |        0 |    100% |           |
| src/phrase\_gym/server/api/study.py        |       31 |        8 |     74% |24-25, 27, 32-35, 41 |
| src/phrase\_gym/server/app.py              |       45 |        0 |    100% |           |
| src/phrase\_gym/server/boundary.py         |       61 |        6 |     90% |72-73, 78, 81-82, 92 |
| src/phrase\_gym/server/contracts.py        |       73 |        2 |     97% |    27, 48 |
| src/phrase\_gym/server/exposure.py         |        6 |        0 |    100% |           |
| src/phrase\_gym/server/importer.py         |       91 |        7 |     92% |36, 75, 104, 119, 123, 128, 154 |
| src/phrase\_gym/server/library.py          |       45 |        1 |     98% |        57 |
| src/phrase\_gym/server/main.py             |       41 |        7 |     83% |16-20, 24, 51 |
| src/phrase\_gym/server/publication.py      |       30 |        1 |     97% |        33 |
| src/phrase\_gym/server/reference\_audio.py |       42 |        3 |     93% |51, 53, 55 |
| src/phrase\_gym/server/scheduling.py       |       46 |        4 |     91% |73, 76, 78, 87 |
| src/phrase\_gym/server/storage.py          |       78 |        2 |     97% |    68, 75 |
| src/phrase\_gym/server/study.py            |      104 |        5 |     95% |45, 132, 152, 174, 181 |
| src/phrase\_gym/server/study\_views.py     |       25 |        0 |    100% |           |
| **TOTAL**                                  | **1071** |  **119** | **89%** |           |


## Setup coverage badge

Below are examples of the badges you can use in your main branch `README` file.

### Direct image

[![Coverage badge](https://raw.githubusercontent.com/andgineer/phrase-gym/python-coverage-comment-action-data/badge.svg)](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

This is the one to use if your repository is private or if you don't want to customize anything.

### [Shields.io](https://shields.io) Json Endpoint

[![Coverage badge](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/andgineer/phrase-gym/python-coverage-comment-action-data/endpoint.json)](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

Using this one will allow you to [customize](https://shields.io/endpoint) the look of your badge.
It won't work with private repositories. It won't be refreshed more than once per five minutes.

### [Shields.io](https://shields.io) Dynamic Badge

[![Coverage badge](https://img.shields.io/badge/dynamic/json?color=brightgreen&label=coverage&query=%24.message&url=https%3A%2F%2Fraw.githubusercontent.com%2Fandgineer%2Fphrase-gym%2Fpython-coverage-comment-action-data%2Fendpoint.json)](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

This one will always be the same color. It won't work for private repos. I'm not even sure why we included it.

## What is that?

This branch is part of the
[python-coverage-comment-action](https://github.com/marketplace/actions/python-coverage-comment)
GitHub Action. All the files in this branch are automatically generated and may be
overwritten at any moment.