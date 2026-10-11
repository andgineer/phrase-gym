[![Build Status](https://github.com/andgineer/phrase-gym/workflows/CI/badge.svg)](https://github.com/andgineer/phrase-gym/actions)
[![Coverage](https://raw.githubusercontent.com/andgineer/phrase-gym/python-coverage-comment-action-data/badge.svg)](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)
# phrase-gym

A language trainer for automatic use of words and phrases through guided recall, spaced repetition, and targeted dialogue drills. 

## Design and implementation handoff

Start with [specs/START-HERE.md](specs/START-HERE.md) for the complete goal, accepted decisions, design discussion, open choices and current status. The [next-session brief](specs/plan/next-session.md) and [roadmap](specs/plan/implementation-plan.md) guide future work.

The app source is still the user-created scaffold. No approved prototype or implemented app is present in active source. Historical experiments are clearly labelled under `specs/archive/`; they do not establish approval or completed milestones. All development documentation lives under `specs/` (plural).

# Documentation

[Phrase Gym](https://andgineer.github.io/phrase-gym/)



# Developers

Do not forget to run `. ./activate.sh`.

For work it need [uv](https://github.com/astral-sh/uv) installed.

Use [pre-commit](https://pre-commit.com/#install) hooks for code quality:

    pre-commit install

## Allure test report

* [Allure report](https://andgineer.github.io/phrase-gym/builds/tests/)

# Scripts
Install [invoke](https://docs.pyinvoke.org/en/stable/) preferably with [uv tool](https://docs.astral.sh/uv/):

    uv tool install invoke

For a list of available scripts run:

    invoke --list

For more information about a script run:

    invoke <script> --help

## Coverage report
* [Codecov](https://app.codecov.io/gh/andgineer/phrase-gym/tree/main/src%2Fphrase_gym)
* [Coveralls](https://coveralls.io/github/andgineer/phrase-gym)

> Created with cookiecutter using [template](https://github.com/andgineer/cookiecutter-python-package)
