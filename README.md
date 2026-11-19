# Phrase Gym

[![CI](https://github.com/andgineer/phrase-gym/workflows/CI/badge.svg)](https://github.com/andgineer/phrase-gym/actions)
[![Coverage](https://raw.githubusercontent.com/andgineer/phrase-gym/python-coverage-comment-action-data/badge.svg)](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

**Make the English you hear part of the English you speak.**

Bring in phrases from a lesson, a video, or a conversation. Get a situation, try saying the phrase
from memory, then hear how it sounds alongside your own take.

<table>
<tr>
<td align="center" valign="top"><b>Say it from a situation</b><br><img src="docs/images/practice-cue.png" width="300" alt="Mobile practice card with a situation cue, hidden answer, and microphone"></td>
<td align="center" valign="top"><b>Reveal, compare, and rate</b><br><img src="docs/images/practice-compare.png" width="300" alt="Revealed phrase with reference and recording playback, comparison, and review intervals"></td>
</tr>
</table>

- **Practise your own material.** Give Coach a phrase or word and it builds a card with
  context, hints, and reference audio. An AI agent can bring in a whole lesson at once.
- **Hear the difference.** Record a take, play the reference and your voice back to back,
  and revisit saved recordings in your Library.
- **Come back when it’s due.** Rate your recall to schedule the next review. Do a short daily
  session, get extra practice when you want it, or pick a phrase to rehearse.

[Install Phrase Gym](https://andgineer.github.io/phrase-gym/installation/) ·
[User guide](https://andgineer.github.io/phrase-gym/) ·
[Add your phrases](https://andgineer.github.io/phrase-gym/importing-phrases/) ·
[Use it on your phone](https://andgineer.github.io/phrase-gym/private-access/)

## Under the hood

**The practice loop comes from research.** A meta-analysis of 48 experiments found that spacing
practice helped second-language learning. Studies of retrieval and spoken production also shaped
the loop. Say the phrase from a situation, compare it with a reference, then recall it again on a
later day. The research notes show how the findings led to those choices and where the evidence
ends.
[Spacing meta-analysis](https://doi.org/10.1111/lang.12479) ·
[Research and design notes](specs/research/README.md)

- **Keeping browser audio tied to the right card.** Closing the microphone after each take on
  desktop Safari interrupted Bluetooth playback. On iPhone, the app releases the input before
  comparison. I built the recorder around those differences and journal each take in IndexedDB
  before advancing. A stable take ID keeps upload retries tied to the original phrase and
  prevents duplicate recordings.
  [Recording design](specs/decision-recording-and-comparison.md)
- **Making AI card quality testable.** A model can return valid JSON and still teach the wrong
  sense or give away the answer. I use the same card rules across model CLIs, check structure
  in code, and review actual model outputs in separate sessions. Coach uses a durable SQLite
  handoff, so the practice app stays usable while the model and local Piper voice prepare audio.
  [Preparation and evaluation](specs/decision-external-preparation-agent.md)
- **Keeping the database layer small by measuring it.** I prototyped SQLAlchemy against the
  real review and search paths. The first version let concurrent writes overwrite each other,
  and the rating save grew from two statements to four. I kept explicit SQLite transactions
  and used typed rows checked against the database schema.
  [Database comparison](specs/architecture.md#database-access)

<details>
<summary><b>Collaborating</b></summary>

Start with the [design decisions and remaining work](specs/START-HERE.md),
[development and releases](specs/development.md), and [agent instructions](AGENTS.md).
See the [Allure test report](https://andgineer.github.io/phrase-gym/builds/tests/) for CI results.

</details>
