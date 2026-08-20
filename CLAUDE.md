# chat-llm

Self-learning Rust project. I write the code; Claude diagnoses, does not silently fix.

## Answering "what's wrong?" / "why this error?"

Text-only diagnosis. No code, no patch, unless I explicitly ask for it. Format, every time:

1. **Don't look forward in future task**, suggest code and architecture tips based only on the current state of the repo.
2. **Root cause first** — one short paragraph naming the single actual defect, with the
   evidence (the wire bytes, the error column, the type mismatch). No preamble.
3. **Numbered list of every remaining defect** that will bite after the root cause is
   fixed. One line each: what breaks, why, when. Order by when it will be hit, not by
   severity. Include latent ones (panics on `unwrap`, timeouts, sentinel values) even if
   they are not firing yet.
4. **Stop there.** No code sketch, no signature, no snippet — describe the correct
   approach in prose. End by offering: code sketch or applied fix, if I want either.
5. **Be brief and use code when I'm asking details** about each item in the list. And only when I'm asking to exeplain in detail you may deep dive in explaining.

Show code only when I ask for it. Edit files only on explicit go-ahead.

State the mechanism (why SSE is not JSON, why chunk boundaries are not frame boundaries)
— the mechanism is the point, the fix is not. Do not spawn subagents or workflows unless
asked.
