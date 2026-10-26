# Repository Coverage

[Full report](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

| Name                                |    Stmts |     Miss |   Cover |   Missing |
|------------------------------------ | -------: | -------: | ------: | --------: |
| src/phrase\_gym/\_\_about\_\_.py    |        1 |        0 |    100% |           |
| src/phrase\_gym/api\_recordings.py  |       50 |        7 |     86% |23-24, 26, 28, 37, 49, 89 |
| src/phrase\_gym/api\_study.py       |       25 |        5 |     80% | 28, 33-36 |
| src/phrase\_gym/app.py              |       51 |       11 |     78% |35, 41-42, 48-49, 53-58 |
| src/phrase\_gym/boundary.py         |       46 |        6 |     87% |42-43, 48, 51-52, 62 |
| src/phrase\_gym/bounded\_process.py |       46 |        3 |     93% | 28, 79-80 |
| src/phrase\_gym/contracts.py        |       55 |        2 |     96% |    27, 48 |
| src/phrase\_gym/importer.py         |      114 |        9 |     92% |54, 94, 96, 98, 104, 144, 157, 188, 204 |
| src/phrase\_gym/main.py             |       19 |       19 |      0% |      1-25 |
| src/phrase\_gym/speech.py           |       87 |        6 |     93% |101, 117, 123, 138, 161-162 |
| src/phrase\_gym/storage.py          |       56 |        0 |    100% |           |
| src/phrase\_gym/study.py            |       72 |        9 |     88% |59, 93, 99, 117-122, 126, 160-161 |
| **TOTAL**                           |  **622** |   **77** | **88%** |           |


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