# Repository Coverage

[Full report](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

| Name                                        |    Stmts |     Miss |   Cover |   Missing |
|-------------------------------------------- | -------: | -------: | ------: | --------: |
| src/phrase\_gym/\_\_about\_\_.py            |        1 |        0 |    100% |           |
| src/phrase\_gym/coach/bounded\_process.py   |       46 |        3 |     93% | 28, 79-80 |
| src/phrase\_gym/coach/main.py               |       59 |       21 |     64% |39-47, 61-62, 69, 76-77, 82-93 |
| src/phrase\_gym/coach/speech.py             |       87 |       10 |     89% |51-52, 101, 117, 123, 138, 161-162, 168-169 |
| src/phrase\_gym/server/api/coach.py         |       25 |        3 |     88% | 23, 31-32 |
| src/phrase\_gym/server/api/deletion.py      |       18 |        0 |    100% |           |
| src/phrase\_gym/server/api/imports.py       |       30 |        4 |     87% |     36-39 |
| src/phrase\_gym/server/api/library.py       |       36 |        1 |     97% |        55 |
| src/phrase\_gym/server/api/recordings.py    |       33 |        5 |     85% |24-25, 27, 29, 41 |
| src/phrase\_gym/server/api/review.py        |       21 |       11 |     48% |12-15, 19-30 |
| src/phrase\_gym/server/api/settings.py      |       15 |        0 |    100% |           |
| src/phrase\_gym/server/api/study.py         |       24 |        6 |     75% |23-24, 31-34 |
| src/phrase\_gym/server/app.py               |       55 |        0 |    100% |           |
| src/phrase\_gym/server/boundary.py          |       61 |        6 |     90% |72-73, 78, 81-82, 92 |
| src/phrase\_gym/server/contracts.py         |       49 |        0 |    100% |           |
| src/phrase\_gym/server/deletion.py          |       29 |        0 |    100% |           |
| src/phrase\_gym/server/exposure.py          |        6 |        0 |    100% |           |
| src/phrase\_gym/server/file\_cleanup.py     |       37 |        0 |    100% |           |
| src/phrase\_gym/server/import\_contracts.py |       91 |        4 |     96% |24, 51, 58, 65 |
| src/phrase\_gym/server/importer.py          |       85 |        6 |     93% |36, 63, 92, 107, 111, 116 |
| src/phrase\_gym/server/lesson\_choices.py   |       27 |        0 |    100% |           |
| src/phrase\_gym/server/library.py           |       38 |        0 |    100% |           |
| src/phrase\_gym/server/main.py              |       46 |        7 |     85% |17-21, 25, 56 |
| src/phrase\_gym/server/phrase\_counts.py    |       11 |        0 |    100% |           |
| src/phrase\_gym/server/publication.py       |       56 |        0 |    100% |           |
| src/phrase\_gym/server/recording\_store.py  |       42 |        2 |     95% |    31, 59 |
| src/phrase\_gym/server/reference\_audio.py  |       42 |        3 |     93% |52, 54, 56 |
| src/phrase\_gym/server/scheduling.py        |       44 |        3 |     93% |73, 79, 88 |
| src/phrase\_gym/server/search\_terms.py     |       10 |        0 |    100% |           |
| src/phrase\_gym/server/settings.py          |       18 |        0 |    100% |           |
| src/phrase\_gym/server/storage.py           |       91 |        2 |     98% |  124, 176 |
| src/phrase\_gym/server/study.py             |       99 |        4 |     96% |130, 150, 172, 183 |
| src/phrase\_gym/server/study\_views.py      |       23 |        0 |    100% |           |
| **TOTAL**                                   | **1355** |  **101** | **93%** |           |


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