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
- 30 Oct 2026: Applying the hackathon's promo code on the billing page failed at first with "Act for contract(...)
  and period(...) not found". Got in the way: the message says nothing the person can act on, and it came with
  hours to go before the deadline.
- 30 Oct 2026: With a real key, the models list offered 30 models, 5 of them Nemotron, under exactly the names in the
  docs. Every reply carried `usage` with the tokens sent and received, so Steplet could count each call's cost as it
  came back. Answers were quick: 0.6 to 2.2 seconds for a short tool call or answer on Super, Ultra and Lightning.
  Helped.

## NVIDIA Nemotron

- 30 Oct 2026: Given one tool (`list_files`) and asked to list a folder, all five Nemotron models offered (3 Super,
  3 Ultra, 3 Nano, 3.5 Lightning, 3.5 Super VL) called it correctly on the first try, with readable arguments, then
  answered from its result without calling it again. Helped: the tool-call shape is the standard one, so Steplet's
  loop needed nothing model-specific.
- 30 Oct 2026: Nemotron 3 Super's thinking comes back in its own field, apart from the answer, and the answer is
  empty on a tool call. Helped: Steplet shows only the answer and the moves, and never has to cut thinking out of it.
- 30 Oct 2026: Nemotron 3 Nano used 1,721 tokens, most of them thinking, to answer a one-line question about a folder
  listing (6.5 seconds). Still the cheapest at $0.0004, but the slowest by far. Got in the way, a little: a cap counted
  in tokens has to leave room for thinking that long.
- 30 Oct 2026: The first whole runs with Nemotron 3 Super in Steplet's agent. Given the sample file, it asked for
  exactly the one command the file allows on its first turn, then reported the result in one true sentence ("5 tests
  ran and all passed."): 2 turns, about $0.0012. Given a file whose text also asks for a command it does not allow, it
  asked for that command, and when Steplet refused it, it did not try another way round: it finished and said plainly
  that it could not report what the refused command would have shown. Helped: it followed the rules it was told, and
  its summary matched Steplet's own record.
- 30 Oct 2026: Asked to describe the sample file as JSON (Steplet's "What will this file do?"), Nemotron 3 Super
  answered in the shape asked, in 3.6 seconds, for $0.00057, within a 1,500-token limit. Helped.
