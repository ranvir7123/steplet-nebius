# Hackathon requirements checklist

Every requirement in the hackathon's official rules, and where Steplet meets it, or that it does not yet. Read on
30 Oct 2026 at about 06:30 India time from the public pages (read only; nothing was signed in to or changed):

- Official rules: https://nebiusglobalaihackathon.devpost.com/rules
- Overview: https://nebiusglobalaihackathon.devpost.com/

Marks: ✅ met · ⏳ not yet, and which task does it · ❓ needs Ranvir's answer or check · ⚠️ a risk to decide on.

## Open questions to settle first

1. ⚠️ **Does Steplet fit the Coding and Agentic Engineering track's wording?** The track reads: "Build coding agents
   and developer tools: agents that write, run, and test code in Token Factory." Token Factory has a separate product
   for running code, **Sandboxes**: "Give applications and AI agents an isolated environment to execute code, evaluate
   results, and work with files." It is in beta, and access has to be requested
   (https://docs.tokenfactory.nebius.com/sandboxes/overview). Steplet's agent gets its model from Token Factory, but
   runs the code in Steplet's own fenced Linux on the laptop, not in a Token Factory Sandbox. Stage One of judging is
   pass/fail on whether "the Project reasonably fits the theme", meaning "a genuine attempt at the track's stated
   goal". Steplet is a coding agent that writes, runs and tests code, and every model call goes to Token Factory, so a
   fair reading lets it in; a strict reading of "in Token Factory" might not. The choices, in plain words:
   - **Keep this track, and say why in the description.** Running a stranger's code on the receiver's own laptop,
     fenced, is the whole point of Steplet; the description already says so. If a judge reads the track strictly,
     the entry could fail Stage One and not be scored at all.
   - **Enter Best Apps and Agents instead.** Its wording ("any app or agent someone would actually use... Power it
     with Nemotron models on Nebius through Token Factory") fits without any doubt. The track prize is the same (one
     Jetson Orin Nano). The cost: the plan, the README and the video were written for the coding track, and their
     track lines would need changing.
   - **Ask the hackathon manager before 22:30 India time** (the overview page has an "Email the hackathon manager"
     link). Only Ranvir can send that; the answer may not come in time.

   The overall prizes ($20,000, $10,000, $6,000) are open to all tracks either way (rules, section 8).

2. ⚠️ **Third-party names in the video.** The rules say the video "must not include third party trademarks, or
   copyrighted music or other material unless the Entrant has permission". The run chooser shows "Claude Code" as the
   second runner. The video script frames shot 3 on Steplet's agent card only, and shot 7 shows the diagram as a plain
   picture rather than a GitHub page. Nebius and NVIDIA are the sponsors.

3. ⚠️ **Licences of the parts built into the installer.** The rules allow open-source software "provided the Entrant
   complies with applicable open source licenses". Steplet's program is built from many open-source Rust and
   JavaScript packages, most under the MIT or Apache licences, which ask that their licence text travel with copies of
   the program. Nothing in the plan adds those texts to the installer or the zip yet. The cost if left: a judge or a
   package's author could say the release does not comply. Fix in Task 12: a file listing each package with its
   licence text, made from `Cargo.lock`, `web/package-lock.json` and the licence files already on this laptop (no
   download).

## Dates

| Requirement (rules section 1) | Where it stands |
|---|---|
| Submission Period: "Wednesday, August 26, 2026 (9:00 am Pacific Time) – Friday, October 30, 2026 (10:00 am Pacific Time)". 10:00 am Pacific on 30 Oct is 22:30 India time. | ⏳ The plan submits by 20:00 India time. |
| Projects started before the period must be "significantly updated" during it. | ✅ Not needed: Steplet's first commit, `18bc9b0`, is dated 8 Oct 2026, 16:37 +05:30, after the period opened. `devpost.md` and the README say so in the agreed sentence. |
| Judging 1 to 15 Dec 2026; winners around 11 Jan 2027. | Nothing to do, except keep the release and repo up (see Testing). |

## Who can enter (rules section 3)

| Requirement | Where it stands |
|---|---|
| At least the age of majority where you live, at the time of entry. | ❓ Ranvir to confirm. |
| Not resident in an excluded place (the rules name Brazil, Quebec, Russia, Crimea, Cuba, Iran, North Korea and other sanctioned places). | ❓ Ranvir to confirm (India is not on the list). |
| Not an employee or family of Nebius, Devpost, a judge, or anyone running the hackathon. | ❓ Ranvir to confirm. |

## How to enter (rules section 4)

| Requirement | Where it stands |
|---|---|
| Register on the hackathon site with "Join Hackathon" and a Devpost account. | ❓ The plan asked Ranvir to do this on 29 Oct; not recorded as done. |
| Fill every required field on the "Enter a Submission" page during the period. | ⏳ From `docs/hackathon/devpost.md`, Fri 18:00 to 20:00. |

## Project requirements (rules section 4)

| Requirement | Where it stands |
|---|---|
| "a working software application that runs on either Nebius Token Factory or Nebius AI Cloud". This "means the project makes a runtime call to the Token Factory inference API". | ⏳ The Token Factory client is built and passes 15 tests against a stand-in server (`crates/steplet-run/tests/tokenfactory.rs`). No real call yet: Task 4 makes the first, Task 10 the first whole runs. |
| "uses at least one NVIDIA open source model". | ⏳ Nemotron 3 Super is the default model in the code (`crates/steplet-run/src/price.rs`); used for real from Task 4. |
| Fits one of the four tracks. | ⚠️ See open question 1. |
| Functionality: "capable of being successfully installed and running consistently on the platform for which it is intended and must function as depicted in the video and/or expressed in the text description." | ⏳ Task 12 installs the release on this laptop and runs use case 1 from it. The video and text must show only what really ran (the video script and `devpost.md` say so). |
| Platforms: runs on the platform it is meant for, named in the submission. | ✅ in the draft: the README's "What you need" names Windows 11 with WSL 2. Tested on one laptop only, which the README says. |
| Third-party SDKs, APIs and data used under their terms. | ❓ Token Factory is used with Ranvir's own account and key, under the terms accepted at sign-up. The Claude Code runner uses the receiver's own Anthropic key. No other service. |

## What to submit (rules section 4, "Submission Requirements")

| Requirement | Where it stands |
|---|---|
| "Provide a URL to a working demo, hosted application, or test build". | ⏳ Task 12: the GitHub release v0.2.0 with the installer, the zip and their SHA-256 sums. Placeholder in `devpost.md`. |
| "a text description that should explain the features and functionality". | ✅ drafted: `docs/hackathon/devpost.md`, with TODOs for Tasks 4 and 7 to 11. |
| "a URL to your public code repository ... on either GitHub, GitLab or Bitbucket". | ⏳ Task 12: `ranvir7123/steplet-nebius`, public, needs Ranvir's yes. The secret scan (`docs/hackathon/secret-scan.md`) found 0 real keys in the history at `2b22b56`; it must run again just before the push. |
| "The repository must contain all necessary source code, assets, and instructions required for the project to be functional." | ⏳ The whole history is pushed (N6). The README's build steps were taken from earlier results and must be run from a clean checkout in Task 12. |
| "public and open source by including an open source license file ... This license should be detectable and visible at the top of the repository page (in the About section)." | ✅ in this branch: `LICENSE` is the Apache 2.0 text, copied byte for byte from a cached crate (SHA-256 `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4`, the same in 4 crates). ⏳ Whether GitHub shows it in the About section can only be seen after the push (Task 12). |
| "Include a README with setup instructions and clear guidance for running your project." | ✅ drafted: `README.md`, with TODOs for the release's file names and for steps checked from the installed app. |
| "highlight how you've used NVIDIA Nemotron ... where Token Factory accelerated your workflow and any other Nebius Tools or Services used." | ✅ drafted: the README's "Steplet's own agent" section (including "Where Token Factory helped" and "No other Nebius service is used"), and `devpost.md`'s "How it uses Nemotron and Token Factory". |
| Video "should be less than three (3) minutes"; judges need not watch past three minutes. | ✅ planned: `docs/hackathon/video-script.md` ends at 2:45, with 270 spoken words. ⏳ Recording Fri 16:00 to 18:00. |
| Video "should include footage that shows the Project functioning on the device for which it was built". | ✅ planned: every shot is the real app on this laptop. Shots 4 to 6 depend on Tasks 7 to 10. |
| Video "must be uploaded to and made publicly visible on YouTube" with the link on the form. | ⏳ Ranvir uploads it. Set it to Public. |
| Video must not include third-party trademarks or copyrighted music without permission. | ✅ no music. ⚠️ See open question 2. |
| "Identify which track you are submitting your Project into." | ✅ `devpost.md`: Coding and Agentic Engineering. ⚠️ See open question 1. |
| "Provide feedback on Nebius Token Factory, AI Cloud and any NVIDIA tools, models, or technologies used." | ⏳ `devpost.md` has the feedback section, from `docs/nebius-feedback.md`. It has one entry so far; Tasks 4 and 10 add what the real calls show. (Ten entrants win $100 and an NVIDIA swag pack for the most valuable feedback, judged on "completeness, viability, and potential impact".) |
| Explain what was updated, if the project existed before the period. | ✅ Not needed (see Dates); the sentence is there anyway. |
| "Be the original work of the Entrant, be solely owned by the Entrant, and not violate the IP rights of any other person or entity." | ❓ Ranvir to confirm. Open-source parts are allowed if their licences are followed (see open question 3). |

## Testing (rules section 4)

| Requirement | Where it stands |
|---|---|
| Access to the working project "by providing a link to a website, functioning demo, or a test build", free and "without any restriction" until judging ends (15 Dec 2026). | ⏳ The release stays public until then. Judges need Windows 11 with WSL 2 and their own Nebius key; the README says so. Steplet never ships a key. Judges "may choose to judge based solely on the text description, images, and video". |
| All materials in English. | ✅ |

## Other rules

| Rule | Where it stands |
|---|---|
| No "financial or preferential support from the Sponsor or Administrator" (funding, contract, commercial licence). | ❓ The Nebius Builder Program credits are offered to every entrant in the rules themselves (section 4). Ranvir to confirm there was no other support. |
| One Overall Award, or one Track Award plus one Bonus Award, per project. | Nothing to do. The Tavily bonus is not entered (Extra 2 is out of the plan). The city prizes are only for those who attended a Builders & Brews event. |
| Submission can't change after the period ends; drafts can be saved before. | Save a draft on Devpost early, then submit before 22:30. |

## Judging criteria (rules section 6), and where the entry speaks to each

Stage One is pass/fail: fits the track, and "reasonably applies the required APIs/SDKs". Stage Two scores four equally
weighted criteria.

| Criterion | Where the entry shows it |
|---|---|
| Technological Implementation: "How well is the project built, and how effectively does it use Nebius Token Factory or AI Cloud model(s), and NVIDIA Nemotron". | The README's tested list; the cap checked before each call; the video's shot 7. ⏳ Real runs (Task 10). |
| Design: "a complete, coherent product experience not just a technical proof of concept". | Preview, chooser, live run, question, report: video shots 2 to 6. ⏳ Tasks 7 and 8. |
| Potential Impact: "a credible, specific case for solving a real problem for a real audience". | The README's "The problem"; `devpost.md`'s "Why we built it". |
| Quality of the Idea: "a creative, non-obvious use of ... Token Factory ... and NVIDIA Nemotron". | Nemotron proposes, a checker it cannot talk to decides, a fence contains; the report puts Steplet's exit codes beside the model's claims. |
