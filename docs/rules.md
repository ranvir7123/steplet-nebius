# Steplet's rating rules

This page is written by `cargo test -p steplet-core --test rules_doc` from `crates/steplet-core/src/rules.rs` (decision G3); do not edit it by hand.

A file's colour is its worst finding (decision 21). **Blocked:** the file cannot run. **Red:** it runs only after the receiver types a confirmation sentence. **Amber:** continue only if you trust the sender. **Note:** shown; the file stays green. Rule ids never change.

The phrase rules follow ideas from Cisco's skill-scanner (Apache-2.0, https://github.com/cisco-ai-defense/skill-scanner, read at commit f18074a5), rewritten as Steplet's own word lists.

| Rule | Severity | What it means | What to do |
|---|---|---|---|
| `read.refused` | blocked | Steplet cannot read this file | Ask the sender to fix the line named here; steplet-sign check shows them what is allowed. |
| `read.too-large` | blocked | The file is bigger than Steplet allows | Everything must fit on your screen. The sender can split the task into several files. |
| `read.line-too-long` | blocked | A line of the header is longer than Steplet allows | The sender can shorten it and put the details in the body. |
| `sig.invalid` | blocked | The signature does not match this file | The file was changed after it was signed, or the signature was damaged. Ask the sender for a fresh copy. |
| `time.expired` | blocked | The file has expired | Ask the sender for a new file with a later expiry date. |
| `time.too-far` | blocked | The file expires more than 365 days from today | Files may be valid for at most 365 days. Ask the sender for a file with an earlier expiry date. |
| `block.secret-path` | blocked | The file names a place where keys or passwords are kept | No task may touch keys, passwords, wallets or sign-ins. Senders: if this is a warning for the agent, put it in the never list. |
| `block.admin` | blocked | A command needs administrator rights | Steplet never gives a task administrator rights. Senders: remove sudo or similar from the command. |
| `block.download-into-shell` | blocked | The file downloads a script and runs it straight away | This is never allowed, even from an official site. Senders: use a package manager, or put the script in the repository. |
| `block.registry` | blocked | A command points a package manager at another source | Steplet does not let a task change where packages come from. Senders: use the normal registry. |
| `block.invisible-in-field` | blocked | An invisible character in the title, the sender or a command | Invisible characters can make a name or a command look different from what it is. Senders: remove them. |
| `block.write-protected` | blocked | The file asks to change .git or an agent's settings | No task may change .git, .claude, .codex, .gemini, .cursor or .mcp.json. Senders: remove it from allow: write. |
| `hide.invisible-in-text` | red | Invisible characters in the text | They can hide words from you that the agent still reads. Each one is shown as a badge; check them before running. |
| `hide.html-comment` | red | An HTML comment in the text | Many Markdown viewers hide HTML comments, but the agent reads them. Steplet shows them; read them before running. |
| `hide.encoded-block` | red | A long block of encoded characters | Encoded text (base64 or hex) can hide instructions or a program. Ask the sender what it is. |
| `hide.blank-run` | red | Many empty lines in a row | A long run of empty lines can push text out of view. Scroll through the whole body before running. |
| `hide.link-mismatch` | red | A link shows one address but opens another | The text of the link names a different site from the one it opens. Steplet shows the real address. |
| `hide.punycode` | red | A web address written in another alphabet | A name with xn-- stands for letters of another alphabet, which can imitate a well-known site. |
| `social.override` | red | The text tells the agent to ignore its earlier instructions | This is a common way to take over an agent. Run it only if you trust the sender completely. |
| `social.conceal` | red | The text tells the agent to hide something from you | An honest task has nothing to hide from the person running it. |
| `social.disable-safety` | red | The text tells the agent to switch off its safety rules | An honest task does not need the agent's safety rules switched off. |
| `social.claims-approval` | red | The text says the run was already approved | Only you approve a run, on Steplet's own screens. Text in a file cannot approve anything. |
| `social.asks-secrets` | red | The text asks for a password, a key or a token | A task like this never needs your passwords, keys or tokens. |
| `social.asks-admin` | red | The text asks for administrator rights | Steplet never gives a task administrator rights, so a step that needs them will fail. |
| `social.disable-protection` | red | The text asks to switch off antivirus, the firewall or other protection | An honest task does not need your computer's protection switched off. |
| `social.lookalike-host` | red | A web address that imitates a well-known site | The name is one letter away from a well-known site, or has a different ending. Check it carefully. |
| `net.exfil-domain` | red | The file lets data go to a paste, tunnel or webhook site | These sites are often used to collect stolen data. A test task rarely needs them. |
| `cap.runs-code` | amber | A command runs the repository's own code | Tests, builds and scripts run code from the repository. Steplet runs them only inside the fence. |
| `cap.installs` | amber | A command downloads and installs packages | Packages are other people's code and can run while they install. Steplet runs this only inside the fence. |
| `cap.unknown-program` | amber | A command uses a program Steplet does not know | Steplet treats it as able to run any code, so it runs only inside the fence. |
| `cap.not-understood` | amber | Steplet cannot read a command exactly | Commands are matched word for word, so the agent will have to ask you each time. Senders: write one plain command, without pipes, semicolons, &&, $, redirections or wildcards. |
| `cap.outside-path` | amber | A command names a path outside the repository | Programs inside the fence can still read some places outside the repository. Check that this one is needed. |
| `net.other-domain` | amber | The file lets the agent and its programs reach a web site | Data can be sent to every site on this list. Package registries Steplet knows are not shown here. |
| `consist.undeclared-command` | amber | The instructions mention a command the file does not ask for | The agent will have to ask you before running it. Senders: add it to allow: commands. |
| `consist.undeclared-host` | amber | A step mentions a web site the file does not ask for | The agent will have to ask you before opening it. Senders: add it to allow: domains. |
| `note.no-repository` | note | The file names no repository | Steplet 1 runs only tasks on a public repository, so this file can be read but not run. |

## Rules about this computer, applied by the app only

These rules need what this computer knows, such as the senders it remembers, so only the desktop app applies them; the public check page does not (decision R10). No file alone can hit them.

| Rule | Severity | What it means | What to do |
|---|---|---|---|
| `id.key-changed` | red | This sender name was used before with another key | Someone may be using this name. Ask the sender, another way, whether they changed their key. If they did, type the sentence to run it; Steplet then remembers the new key. |
