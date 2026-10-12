# Repository Coverage

[Full report](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

| Name                                                          |    Stmts |     Miss |   Cover |   Missing |
|-------------------------------------------------------------- | -------: | -------: | ------: | --------: |
| src/phrase\_gym/\_\_about\_\_.py                              |        1 |        0 |    100% |           |
| src/phrase\_gym/api/app.py                                    |       59 |        9 |     85% | 65-73, 81 |
| src/phrase\_gym/api/boundary.py                               |       50 |        0 |    100% |           |
| src/phrase\_gym/api/exposures.py                              |        6 |        0 |    100% |           |
| src/phrase\_gym/api/imports.py                                |       18 |        0 |    100% |           |
| src/phrase\_gym/api/library.py                                |       25 |        0 |    100% |           |
| src/phrase\_gym/api/media.py                                  |       18 |        1 |     94% |        18 |
| src/phrase\_gym/api/parameters.py                             |        5 |        0 |    100% |           |
| src/phrase\_gym/api/sessions.py                               |       16 |        0 |    100% |           |
| src/phrase\_gym/api/settings.py                               |        9 |        0 |    100% |           |
| src/phrase\_gym/clock.py                                      |        3 |        0 |    100% |           |
| src/phrase\_gym/config.py                                     |       15 |        0 |    100% |           |
| src/phrase\_gym/content/catalog.py                            |       71 |        1 |     99% |        69 |
| src/phrase\_gym/content/exposures.py                          |       24 |        1 |     96% |        25 |
| src/phrase\_gym/content/identity.py                           |       18 |        0 |    100% |           |
| src/phrase\_gym/content/imports.py                            |       55 |        3 |     95% |51, 58, 65 |
| src/phrase\_gym/content/publication.py                        |       49 |        0 |    100% |           |
| src/phrase\_gym/content/validation.py                         |       51 |        0 |    100% |           |
| src/phrase\_gym/contracts/base.py                             |        7 |        0 |    100% |           |
| src/phrase\_gym/contracts/content.py                          |       75 |        0 |    100% |           |
| src/phrase\_gym/contracts/exposures.py                        |        7 |        0 |    100% |           |
| src/phrase\_gym/contracts/imports.py                          |        9 |        0 |    100% |           |
| src/phrase\_gym/contracts/library.py                          |       10 |        0 |    100% |           |
| src/phrase\_gym/contracts/media.py                            |       24 |        0 |    100% |           |
| src/phrase\_gym/contracts/sessions.py                         |       20 |        0 |    100% |           |
| src/phrase\_gym/contracts/settings.py                         |        7 |        0 |    100% |           |
| src/phrase\_gym/errors.py                                     |        4 |        0 |    100% |           |
| src/phrase\_gym/main.py                                       |       96 |       22 |     77% |47-48, 70-72, 85, 92-93, 114-131, 138-143 |
| src/phrase\_gym/media/service.py                              |      106 |        7 |     93% |32, 43, 126-130 |
| src/phrase\_gym/migrations/env.py                             |        5 |        0 |    100% |           |
| src/phrase\_gym/migrations/versions/0001\_foundation.py       |       20 |        2 |     90% |    95-107 |
| src/phrase\_gym/migrations/versions/0002\_device\_language.py |       16 |        1 |     94% |        27 |
| src/phrase\_gym/migrations/versions/0003\_sessions.py         |       12 |        2 |     83% |     39-40 |
| src/phrase\_gym/serialization.py                              |        3 |        0 |    100% |           |
| src/phrase\_gym/sessions/planning.py                          |       40 |        1 |     98% |        60 |
| src/phrase\_gym/sessions/service.py                           |       70 |        0 |    100% |           |
| src/phrase\_gym/settings.py                                   |       13 |        0 |    100% |           |
| src/phrase\_gym/storage.py                                    |      102 |        0 |    100% |           |
| **TOTAL**                                                     | **1139** |   **50** | **96%** |           |


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