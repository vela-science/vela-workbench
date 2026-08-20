# Phase 1 inheritance benchmark — dormant execution plan

Status: **not authorized to run**. The independent Phase 0 review must pass
before this plan may be frozen as a preregistration. Benchmark task
`01a01da2-e354-7453-a60b-aa044d111971` is the sole owner of fixtures, runner,
scoring and review. Workbench must not implement or duplicate them.

## Phase 0 binding evidence

- preregistration SHA-256:
  `025fb027f948323afda33a846eb9f07dcf78a56f980b6e54fcfbc6728180bbc8`;
- candidate-output root:
  `sha256:563e5c5d0d785868de169ffea348dbb50d257936075c6bae99c6fe72f2caa2a4`;
- 18 assigned sessions, 17 valid and one retained infrastructure failure;
- the failed Erdős 321/native/GPT-5.6-sol session ran 1,371.618s, returned no
  answer and was not retried; and
- the preregistration omitted a numeric timeout ceiling. Phase 1 must not repeat
  that defect.

The scored Phase 0 values remain descriptive pending independent review. They
must not be used as claim-grade lift, a product-release gate, or a reason to
unfreeze Core or Workbench.

## Input roster

Phase 1 adds the seven cases not scored in Phase 0. The exact sources and rights
are frozen in [RESULTS_FIRST.md](RESULTS_FIRST.md).

| Block | Case | Role in the scale-up |
|---|---|---|
| P1-04 | Paul batch #8, Erdős 461 at `fc772f15aa14e01a081c4bdb6a7472f2ea95e65f` | CLEAN quality-control and cold-successor control |
| P1-05 | Paul batch #9, Erdős 545 at `f11eee52f0189ec222247f855397006fb1cd8f4e` | source-scope and correction case |
| P1-06 | Paul batch #10, Erdős 550 at `735440a72e3b0f1dbb25b83682438cecc3b6476f` | CLEAN quality-control and cold-successor control |
| P1-07 | Hadamard 668 wrapper `da94bc80401b6ece36d8dd2f5c316755fd97dd65` | evidence, trust and exact-replay case |
| P1-08 | Erdős 94 proof `423344341fbfdf4f8f684a302c5d05379125e7dc` | bounded scope, correction and custody case |
| P1-09 | FAR/ProbXiv Ramsey locator plus FAR `0f498a7e9252affd478cbfe324f51ea6d0119331` | discovery and accountable-producer gap |
| P1-10 | Directed 3-torus source `753cbe37dc6428b15f5109b801301115ec61eb5d` | formal-check versus source-fidelity case |

ProbXiv contributes locators and short observed facts only. Its page bodies are
not reusable inputs because no public content licence or export contract was
observed.

## Proposed paired matrix

For each of seven cases, freeze the same complete information once into the
three Phase 0 views: raw Git/source, native source/registry and Vela package.
Run `gpt-5.6-sol` and `gpt-5.4`, the same two model families used in Phase 0,
with one cold session per cell:

```text
7 cases × 3 views × 2 models = 42 assigned candidate sessions
```

There are no discretionary retries. A timeout or provider failure remains in
the assigned-denominator analysis with no answer, and the report must also show
paired-block sensitivity excluding the entire affected case/model block. The
preregistration must state a **300-second per-session wall-clock ceiling** and
the exact process-termination rule before any run.

Dual treatment-blind judges score the seven frozen questions for each assigned
session. Judge identity, replacement rule, timeout and manual-resolution rule
must be preregistered. A replacement judge may be used only under that frozen
rule and remains a named limitation.

## Compute and cost envelope

This is an authorization ceiling, not expected consumption:

| Work | Calls | Per-call ceiling | Aggregate ceiling |
|---|---:|---:|---:|
| Candidate sessions | 42 | 75,000 input tokens; 4,000 output tokens; 20 tool calls; 300s | 3,150,000 input; 168,000 output; 840 tool calls; 3.5 serial hours |
| Blind judging | 84 | 20,000 input tokens; 2,000 output tokens; no tools | 1,680,000 input; 168,000 output |
| Total | 126 | — | 4,830,000 input; 336,000 output |

No dollar amount is frozen here because provider/model prices can change. At
authorization time the benchmark owner must substitute the current prices into
the token ceilings, publish the calculation, set a hard aggregate dollar limit,
and obtain explicit approval. The run must not start unless timeout and budget
enforcement are observable and fail closed. No provider configuration change is
authorized by this plan.

## Scale-up decision

Run only if the independent Phase 0 review confirms the frozen roots, leakage
audit, scoring, treatment-blind adjudication and declared deviations. Phase 1
must be rejected or amended before preregistration if information equality
cannot be proved for every new block, rights do not permit the retained bytes,
or the execution host cannot enforce the numeric timeout and aggregate budget.
