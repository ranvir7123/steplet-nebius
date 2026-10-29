# Feedback for Nebius and NVIDIA

What building Steplet's own agent on Nebius Token Factory and NVIDIA Nemotron was like, written down as it happened.
Each line has the date, what was seen, and whether it helped or got in the way. Only things actually seen are listed;
the Devpost feedback section is written from this list.

## Nebius Token Factory

- 29 Oct 2026: The API follows the OpenAI chat-completions shape (`/v1/chat/completions`, `tools`, `tool_choice`,
  `usage`). Helped: Steplet talks to it with Windows' own web library and about 100 lines of its own code; no
  Nebius-specific package or SDK (software kit) was needed.

## NVIDIA Nemotron

- Nothing yet: the first real calls are Task 4.
