# Spacing and grading

Question: when should a phrase come back, and what should the learner's rating mean? The current policy lives in [the core trainer decision](../decision-core-trainer.md).

## Spacing

- **Kim & Webb (2022)**, [The effects of spaced practice on second language learning: a meta-analysis](https://doi.org/10.1111/lang.12479). Checked: abstract. 48 experiments, 3,411 participants. Spacing benefited L2 learning, and longer spacing favored delayed outcomes. Equal and expanding schedules were comparable overall. **For us:** spaced returns; no fixed expanding ladder is uniquely best.
- **Macis, Sonbul & Alharbi (2021)**, [Spacing of incidental and deliberate collocation learning](https://doi.org/10.1016/j.system.2021.102649). Checked: abstract. Spacing had a large effect on deliberate collocation learning (form recall after three weeks) and a small effect on incidental learning.
- **Sonbul, Macis & Gyllstad (2025)**, [Equal versus expanding spacing for L2 collocations](https://doi.org/10.1002/tesq.3364). Checked: abstract. 83 learners, 23 collocations. A 7–7–7-day schedule and a 1–5–15-day schedule both beat control and did not differ on recall; recognition favored equal spacing.
- **Eglington & Pavlik (2020)**, [Optimizing practice scheduling requires quantitative tracking of individual item performance](https://pmc.ncbi.nlm.nih.gov/articles/PMC7567101/). Checked: full text. Scheduling items at modeled recall thresholds of .86 and .94 outperformed .40, .98 and a conventional comparator, counting both success and time cost. **For us:** adaptive per-item scheduling at high but not maximal predicted recall. Limit: word pairs; not FSRS.
- **Settles & Meeder (2016)**, [A trainable spaced repetition model for language learning](https://aclanthology.org/P16-1174/). Checked: full text. Learned memory models predicted recall better than baselines. Engagement metrics in the online trial are not memory outcomes.
- **FSRS** ([py-fsrs](https://github.com/open-spaced-repetition/py-fsrs), [tutorial](https://github.com/open-spaced-repetition/fsrs4anki/blob/main/docs/tutorial.md)) documents software behavior, not learning outcomes. The tutorial warns against rating forgotten answers as Hard.

## Retrieval and repetition

- **Karpicke & Roediger (2008)**, [The critical importance of retrieval for learning](https://doi.org/10.1126/science.1152408). Checked: abstract. Continued retrieval after the first correct answer improved delayed recall; continued study did not. Learners' predictions did not track their performance. **For us:** keep testing after early success, and rate the attempt itself, not how familiar the answer feels once revealed.
- **Nakata (2017)**, [Does repeated practice make perfect?](https://doi.org/10.1017/S0272263116000280). Checked: abstract. More retrievals within a session raised delayed scores, but per minute spent, a single retrieval was most efficient. **For us:** bounded repetition within a session, no endless retries.
- **Rawson & Dunlosky (2011)**, [Optimizing schedules of retrieval practice](https://pubmed.ncbi.nlm.nih.gov/21707204/). Checked: abstract. Relearning across sessions strongly improved retention after one to four months, while a higher initial criterion mattered less once relearning followed. **For us:** successful retrieval again on later days matters more than drilling to a high criterion on day one.
- **Yanagisawa (2016)**, [Receptive and productive word retrieval practice](https://www.jstage.jst.go.jp/article/katejournal/30/0/30_11/_article/-char/en). Checked: abstract. In a small study (18 learners), recalling word forms helped productive tests more than recalling meanings did. **For us:** practise the direction that is needed.

## Self-rating

- **Dunlosky & Rawson (2012)**, [Overconfidence produces underachievement](https://eric.ed.gov/?id=EJ964388). Checked: abstract. More accurate judgments of one's own answers went with better retention. **For us:** show the reference and clear criteria, then rate the original attempt. The number of rating buttons is a design choice.
