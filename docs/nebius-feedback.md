# Feedback for Nebius and NVIDIA

What building Steplet's own agent on Nebius Token Factory and NVIDIA Nemotron was like, written down as it happened.
Each line has the date, what was seen, and whether it helped or got in the way. Only things actually seen are listed;
the Devpost feedback section is written from this list.

## Nebius Token Factory

- 29 Oct 2026: The API follows the OpenAI chat-completions shape (`/v1/chat/completions`, `tools`, `tool_choice`,
  `usage`). Helped: Steplet talks to it with Windows' own web library and about 100 lines of its own code; no
  Nebius-specific package or SDK (software kit) was needed.
- 30 Oct 2026: The first real calls, with no key and with a made-up one (they cost nothing). Both came back as HTTP
  401 with a short reason: "token is not present" with no key, "Unable authenticate" with a made-up one. Helped:
  Steplet tells "the key is wrong" apart from every other failure by the status alone, and says so in plain words.
  Got in the way, a little: the list of models also needs a key, so Steplet cannot check a model name is offered
  before the owner has typed in a key.

## NVIDIA Nemotron

- Nothing yet: the first real calls are Task 4.
