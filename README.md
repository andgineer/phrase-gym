# Repository Coverage

[Full report](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

| Name                                                |    Stmts |     Miss |   Cover |   Missing |
|---------------------------------------------------- | -------: | -------: | ------: | --------: |
| src/phrase\_gym/\_\_about\_\_.py                    |        1 |        0 |    100% |           |
| src/phrase\_gym/coach/agents/ai\_agent.py           |      118 |        8 |     93% |21-22, 58, 134-135, 144, 229-230 |
| src/phrase\_gym/coach/agents/antigravity.py         |       89 |        8 |     91% |38, 85, 115-116, 140-143 |
| src/phrase\_gym/coach/agents/antigravity\_errors.py |       68 |       10 |     85% |14-15, 39, 56, 128-129, 132-134, 139 |
| src/phrase\_gym/coach/agents/command\_template.py   |       72 |       14 |     81% |18, 29, 41-42, 49, 66-67, 86, 88, 92, 95, 106-107, 109 |
| src/phrase\_gym/coach/agents/config.py              |        9 |        0 |    100% |           |
| src/phrase\_gym/coach/agents/subprocess.py          |       94 |       22 |     77% |119-134, 137-146, 149-151, 157-158, 174, 179-180, 194-195, 198-203 |
| src/phrase\_gym/coach/basic\_card.py                |       61 |        1 |     98% |       103 |
| src/phrase\_gym/coach/bounded\_process.py           |       46 |        3 |     93% | 28, 79-80 |
| src/phrase\_gym/coach/card\_checks.py               |       53 |        1 |     98% |        64 |
| src/phrase\_gym/coach/claude\_cli.py                |       33 |        2 |     94% |     68-69 |
| src/phrase\_gym/coach/cli\_steps.py                 |       52 |        5 |     90% |70, 79-80, 86, 92 |
| src/phrase\_gym/coach/codex\_cli.py                 |       47 |        1 |     98% |       105 |
| src/phrase\_gym/coach/companion\_checks.py          |       51 |        0 |    100% |           |
| src/phrase\_gym/coach/gemini\_cli.py                |       52 |        0 |    100% |           |
| src/phrase\_gym/coach/hint\_checks.py               |      107 |        1 |     99% |        58 |
| src/phrase\_gym/coach/main.py                       |       95 |       29 |     69% |46-54, 66-67, 71-72, 94-104, 121, 133-145 |
| src/phrase\_gym/coach/messages.py                   |        6 |        0 |    100% |           |
| src/phrase\_gym/coach/preparation.py                |       77 |        3 |     96% |63, 70, 120 |
| src/phrase\_gym/coach/recognition.py                |       45 |        0 |    100% |           |
| src/phrase\_gym/coach/speech.py                     |       87 |        8 |     91% |99, 115, 121, 136, 159-160, 166-167 |
| src/phrase\_gym/server/allowance.py                 |       27 |        0 |    100% |           |
| src/phrase\_gym/server/api/coach.py                 |       55 |        3 |     95% |     54-56 |
| src/phrase\_gym/server/api/deletion.py              |       18 |        0 |    100% |           |
| src/phrase\_gym/server/api/facts.py                 |       16 |        0 |    100% |           |
| src/phrase\_gym/server/api/imports.py               |       25 |        0 |    100% |           |
| src/phrase\_gym/server/api/library.py               |       36 |        1 |     97% |        61 |
| src/phrase\_gym/server/api/paging.py                |        5 |        0 |    100% |           |
| src/phrase\_gym/server/api/practice.py              |       68 |        0 |    100% |           |
| src/phrase\_gym/server/api/recordings.py            |       33 |        5 |     85% |24-25, 27, 29, 41 |
| src/phrase\_gym/server/api/settings.py              |       18 |        2 |     89% |     22-23 |
| src/phrase\_gym/server/app.py                       |       78 |        0 |    100% |           |
| src/phrase\_gym/server/boundary.py                  |       61 |        6 |     90% |72-73, 78, 81-82, 92 |
| src/phrase\_gym/server/coach\_cards.py              |       23 |        1 |     96% |        66 |
| src/phrase\_gym/server/coach\_contracts.py          |       53 |        1 |     98% |        60 |
| src/phrase\_gym/server/coach\_inbox.py              |       29 |        0 |    100% |           |
| src/phrase\_gym/server/coach\_requests.py           |       99 |        3 |     97% |96, 101, 169 |
| src/phrase\_gym/server/completion.py                |       90 |        1 |     99% |       118 |
| src/phrase\_gym/server/contracts.py                 |       60 |        0 |    100% |           |
| src/phrase\_gym/server/current\_practice.py         |       25 |        0 |    100% |           |
| src/phrase\_gym/server/daily.py                     |       30 |        0 |    100% |           |
| src/phrase\_gym/server/deletion.py                  |       32 |        0 |    100% |           |
| src/phrase\_gym/server/extra.py                     |       34 |        0 |    100% |           |
| src/phrase\_gym/server/facts\_contracts.py          |       14 |        0 |    100% |           |
| src/phrase\_gym/server/file\_cleanup.py             |       44 |        0 |    100% |           |
| src/phrase\_gym/server/import\_contracts.py         |       81 |        3 |     96% |30, 57, 64 |
| src/phrase\_gym/server/import\_notices.py           |       16 |        0 |    100% |           |
| src/phrase\_gym/server/importer.py                  |       54 |        1 |     98% |        79 |
| src/phrase\_gym/server/languages.py                 |        5 |        0 |    100% |           |
| src/phrase\_gym/server/lesson\_choices.py           |       26 |        0 |    100% |           |
| src/phrase\_gym/server/library.py                   |       42 |        0 |    100% |           |
| src/phrase\_gym/server/main.py                      |       47 |        7 |     85% |17-21, 25, 62 |
| src/phrase\_gym/server/phrase\_counts.py            |       12 |        0 |    100% |           |
| src/phrase\_gym/server/phrase\_practice.py          |       30 |        0 |    100% |           |
| src/phrase\_gym/server/practice\_contracts.py       |       30 |        0 |    100% |           |
| src/phrase\_gym/server/practice\_facts.py           |       50 |        0 |    100% |           |
| src/phrase\_gym/server/practice\_selection.py       |       46 |        1 |     98% |        81 |
| src/phrase\_gym/server/practice\_state.py           |       32 |        1 |     97% |        57 |
| src/phrase\_gym/server/practice\_today.py           |       44 |        1 |     98% |        71 |
| src/phrase\_gym/server/practice\_views.py           |       59 |        1 |     98% |       149 |
| src/phrase\_gym/server/publication.py               |       64 |        0 |    100% |           |
| src/phrase\_gym/server/recording\_store.py          |       50 |        2 |     96% |    42, 71 |
| src/phrase\_gym/server/recording\_views.py          |       12 |        0 |    100% |           |
| src/phrase\_gym/server/records.py                   |       29 |        0 |    100% |           |
| src/phrase\_gym/server/reference\_audio.py          |       42 |        6 |     86% |23-25, 52, 54, 56 |
| src/phrase\_gym/server/scheduling.py                |       59 |        1 |     98% |        48 |
| src/phrase\_gym/server/schema\_upgrade.py           |       38 |        0 |    100% |           |
| src/phrase\_gym/server/search\_terms.py             |       10 |        0 |    100% |           |
| src/phrase\_gym/server/sentence\_gaps.py            |       29 |        2 |     93% |     38-39 |
| src/phrase\_gym/server/settings.py                  |       27 |        2 |     93% |     23-24 |
| src/phrase\_gym/server/settings\_store.py           |       14 |        0 |    100% |           |
| src/phrase\_gym/server/storage.py                   |      102 |        2 |     98% |   74, 145 |
| src/phrase\_gym/server/streak.py                    |       23 |        0 |    100% |           |
| src/phrase\_gym/server/view\_actions.py             |       53 |        0 |    100% |           |
| **TOTAL**                                           | **3362** |  **169** | **95%** |           |


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