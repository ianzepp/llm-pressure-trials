# Provenance

This repository was assembled on 2026-10-16 from two older repositories,
using `git filter-repo`. This file records how each claim about dates and
data integrity can be checked, and how strong each piece of evidence is.

**Caveat.** Git commit timestamps are set by the committer, so no single
layer below is proof on its own. The case rests on several independent
records agreeing with each other.

## 1. Git history (kept through the filter)

Author and committer dates are preserved. Original hashes map to the
rewritten ones as follows:

| Source repo | Original | Here | Date | Subject |
| --- | --- | --- | --- | --- |
| faber-trials | `00bef6fd87a4` | `399e2e7` | 2026-01-11 | Initial commit: Split probationes from faber repo |
| faber-trials | `73cd6beb090f` | `6502727` | 2026-01-11 | Remove openai dependency from main compiler |
| faber-trials | `b585db4896f7` | `4403a6c` | 2026-01-20 | Training data |
| faber-trials | `6815a23102a0` | `0b50033` | 2026-01-20 | Ignore .env |
| faber-trials | `645e7bbff58b` | `4502e2e` | 2026-01-24 | Add conversation dynamics research: yarrow, dynamics tasks, session report |
| faber-trials | `907446569a83` | `2dd55e5` | 2026-02-03 | Update faber-trials |
| ethics-trials | `c9bd4c3737e8` | `06247f0` | 2026-03-26 | Initial ethics-trials project skeleton |
| ethics-trials | `d5a89a95526d` | `5fc8a09` | 2026-03-26 | Build Rust runner skeleton with SQLite reporting |
| ethics-trials | `b6d1fbe8677c` | `583e20e` | 2026-03-26 | Add model sync and selection controls |
| ethics-trials | `60b1b4f912a4` | `0d23b14` | 2026-03-26 | Add live run scheduler controls and JSONL event output |
| ethics-trials | `6ce974632d4f` | `d6c1a26` | 2026-03-26 | Handle failed trials and continue runs on error |
| ethics-trials | `1b1ea2be6291` | `dd013ac` | 2026-03-26 | Add hygiene checks and harden runner error handling |
| ethics-trials | `301ce145de69` | `c4d8fc7` | 2026-03-26 | docs: sync docs and add smoke paid cohort |
| ethics-trials | `53d638f55293` | `12a1410` | 2026-03-26 | Add pricing capture, usage stats, and experiment repeats |
| ethics-trials | `917822a5cea7` | `4e4d5bd` | 2026-03-26 | Add mercy dilemma study and grade command for regrading archived results |
| ethics-trials | `3ae3e032b493` | `af2df1a` | 2026-03-26 | Improve grading: broader refusal detection and explicit decision extraction |
| ethics-trials | `8440d36d59cf` | `6326409` | 2026-03-26 | Add coercion follow-up on refusal and community voting in reports |
| ethics-trials | `dd74e84c6ea1` | `f25496c` | 2026-03-26 | Add LLM-as-judge for ambiguous trial grading and broader refusal detection |
| ethics-trials | `6983f217f734` | `e793623` | 2026-03-26 | Add drone doctrine study with configurable decision grading |
| ethics-trials | `2d9e9b374696` | `92647ae` | 2026-03-26 | Add two-stage escalation and alternate framing experiments |
| ethics-trials | `fd9d5d5d356b` | `bb96cf9` | 2026-03-26 | Add configurable temperature support |
| ethics-trials | `c135713f9637` | `826c621` | 2026-05-01 | Add dependency release age policy |

Some kept commit messages mention work that was filtered out (the Faber
compiler, conversation-dynamics research). The files were removed; the
messages are left as they were written.

**What was filtered out of `faber-trials`:** Faber-language translation
tasks and results, the `trials.db` index, `node_modules/`, a `.env` file
(its key was rotated long ago), and the conversation-dynamics and yarrow
work, which includes private transcripts. The January ethics results kept
here are the raw `results/chain-*/chain_results.jsonl` files.

`faber-trials` was public on GitHub from January to 2026-10-16, which
gives outside evidence for the January material. It is now private.

## 2. March run data

The March 2026 run artifacts were gitignored when they were produced, so
they have no commit before 2026-10-16. Evidence for their date:

- `ethics-trials/DATA-MANIFEST.tsv` lists every file with size, on-disk
  mtime, and sha256. All mtimes fall between 2026-03-26T18:23:52Z and
  2026-03-26T23:40:54Z.
- Run ids are the run start time in Unix milliseconds. For example,
  `run-1774557774975` is 2026-03-26T20:42:54Z.
- Each run stores the OpenRouter per-token prices in force at run time
  (`pricing.json`).
- An independent check is the OpenRouter account activity log for
  2026-03-26, which the account owner can export.

## 3. Session transcripts

The sessions that produced this work were captured by `cassio` into a
private transcripts repository, which takes daily git snapshots. These
are the relevant files. Hashes are of the current file content; first
commits are given under the file's earlier name, before an April rename.

| Transcript | sha256 (current) | First committed |
| --- | --- | --- |
| `2026-01/2026-01-20T03-36-57-claude.md` | `432330755de901e928f7d57e55179ad8bba84c62060f029e208619f313bcc9a8` | 2026-01-23 (`f34ea340`) |
| `2026-01/2026-01-20T04-51-36-claude.md` | `e05685639955d58e399b2551bba16fc6756b315b3e808d3d19c1c549dcf6f98c` || 2026-01-23 (`f34ea340`) |
| `2026-03/2026-03-26T12-27-59-codex.md` | `a842ba7716becd308b4c824885d0bf198f73256104c1f89774277b39e93d442c` || 2026-03-27 (`5c06f4af`) |
| `2026-03/2026-03-26T13-07-18-codex.md` | `88ca67ab990316d99cc0ec3d09fbc8193a51e0efb0c11eb20724dbddd84eff93` || 2026-03-27 (`5c06f4af`) |
| `2026-03/2026-03-26T17-23-58-claude.md` | `c17a1930e875dccb38bb10e401961c9fc56bda108e277f1de89330afadc49642` || 2026-03-27 (`5c06f4af`) |
| `2026-03/2026-03-26T19-16-21-claude.md` | `c4ef5767ea228b6e65ed42afe530b741c4078b04018e257d88bdab50fbf927df` | 2026-03-27 (`5c06f4af`) |
| `2026-03/2026-03-26T19-22-12-claude.md` | `4f7068407dc1ba4fab4edd59166753f7539cb5db74adde0e497d124418764378` || 2026-03-27 (`5c06f4af`) |
| `2026-03/2026-03-26T20-03-27-claude.md` | `9b1a75828c13c1eb67164c1e1222f927e5436789a40c68242f6c4b8911d39aee` || 2026-03-27 (`5c06f4af`) |
| `2026-03/2026-03-26T23-31-31-claude.md` | `a04dd776aab4e75b92876029e88a1274f6a4ec47cc156e2a33a141c45ba582dd` || 2026-03-27 (`5c06f4af`) |

The 2026-03-27 commit is the daily cassio snapshot taken the morning
after the sessions. The transcripts
themselves are not published here: they contain unrelated private
material.

## 4. Raw session exports

Each transcript has a matching raw export in `~/personal/training/`
(backed up to the `pharos` server). Each export records the agent's own
session id, project path, client version, per-event timestamps, and a
sha256 of the original agent log it was parsed from. The original logs
are no longer on disk, so those hashes cannot be re-checked today.

| Export | Session id | Project | Client | Started (UTC) | Source log sha256 (prefix) |
| --- | --- | --- | --- | --- | --- |
| `2026-01-20T03-36-57-claude` | `6e523f72-5141-43aa-a22c-0c6e9624acf4` | faber | Claude Code 2.1.12 | 2026-01-20T03:36:57Z | `86c84cb0212dcf01` |
| `2026-01-20T04-51-36-claude` | `6e523f72-5141-43aa-a22c-0c6e9624acf4` | faber | Claude Code 2.1.12 | 2026-01-20T04:51:36Z | `6a750e1dc3e09bb8` |
| `2026-03-26T12-27-59-codex` | `019d2b30-4d00-78e3-891f-e4aaa76db033` | ai-trials | Codex 0.117.0-alpha.12 | 2026-03-26T17:27:59Z | `28a827044f8f765d` |
| `2026-03-26T13-07-18-codex` | `019d2b54-4d26-7070-877a-539f64940920` | ethics-trials | Codex 0.117.0-alpha.12 | 2026-03-26T18:07:18Z | `e5160d5b828d867d` |
| `2026-03-26T17-23-58-claude` | `26e20bde-f1b7-40d6-a26a-32971365e243` | github | Claude Code 2.1.78 | 2026-03-26T17:23:58Z | `8814f196ebf49609` |
| `2026-03-26T19-16-21-claude` | `ce1dd375-e6ad-479f-95b8-fc68446be803` | ethics-trials | Claude Code 2.1.78 | 2026-03-26T19:16:21Z | `d591c0f9d92e1ac1` |
| `2026-03-26T19-22-12-claude` | `ce1dd375-e6ad-479f-95b8-fc68446be803` | ethics-trials | Claude Code 2.1.78 | 2026-03-26T19:22:12Z | `f1f9ca3da7023147` |
| `2026-03-26T20-03-27-claude` | `ce1dd375-e6ad-479f-95b8-fc68446be803` | ethics-trials | Claude Code 2.1.78 | 2026-03-26T20:03:27Z | `8cbafad59f0b65e7` |
| `2026-03-26T23-31-31-claude` | `ce1dd375-e6ad-479f-95b8-fc68446be803` | ethics-trials | Claude Code 2.1.78 | 2026-03-26T23:31:31Z | `23cee5185eee2700` |

Session `ce1dd375…` is the main March 26 build-and-run session. It ran
from 19:16Z, and every original run id (18:23Z to 23:40Z in the data
manifest) falls inside the window covered by these sessions.
