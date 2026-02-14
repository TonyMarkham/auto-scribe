# Efficient Prompt Workflow

### Plan

Use Cluade Code's Panning Mode and allow it to create and itterate on the original plan

### Plan > PM
```text
- use the /pm skill to transfer the plan to the Agile Board.
- Use proper Agile Techniques to organize Epics > Stories > Tasks 
- Use the tasks to show a concrete implementation complete with production-grade code snippets
- The tasks should also be written with well thought out explanations above each code block to give humans well structured context 
```

### Review
```text
- use the /pm skill to list all orphaned WorkItems
- Find PONE-2
- Review all descendant tasks.
- Write the review as a comment on PONE-2
```

### Address Review
```text
- use the /pm skill to list all orphaned WorkItems
- Find PONE-2
- Address the review in the latest comment
- use the /pro-rust skill to align with my preferred patterns
```

### Implement
```text
- use /pm to list all orphaned WorkItems
- Find PONE-2
- Review all descendants
- Move PONE-2 and all descendant to `In Progress`
- read `CRITICAL_OPERATING_CONSTRAINTS.md`
- Teach me by presenting bite-sized chunks for me to write/edit keeping the commentary separate from the code snippets to make the code snippets easier for me to follow
- As WorkItems are completed, use /pm to move them to `review`
```

### Commit
```text
I have staged all files, please commit without a byline.
```

---

---

---

---

---

---

---

---

---

### Init
```txt
Read `docs/implementation-plan-v2.md.` We just finished impl'ing session 60
```

### Plan
```txt
Create a step-by-step implementation plan complete with proposed code snippets for how to implement Session 100.
```

### Challenge
```txt
Evaluate your plan and rate it based on how production grade it is.
```

### Product-Grade
```text
My quality bar is to be at least 9.25 out of 10 on a production grade score.

Iterating on your plan until you meet the target level.
```

### ordering
```text
Verify that the `80-Session-Plan-Teaching.md` has proper dependency ordering so that when implementing sequentially, nothing references code that hasn't been written yet.
```

```text
please stop thinking in terms of sub-sessions and instead in dependency order
```

```text
Can you show a dependency graph?
```

### x
```text
before continuing, can I get you to perform a fresh-eyes review of the plan.

Identify strengths/weaknesses/issues and re-grade x.x/10 for production-grade
```

### pm
```text
- use the /pm skill to transfer the plan to the Agile Board.
- Use proper Agile Techniques to organize Epics > Stories > Tasks 
- Use the tasks to show a concrete implementation complete with production-grade code snippets
- The tasks should also be written with well thought out explanations above each code block to give humans well structured context
- Do not try to automate this, perform each interaction 1 at a time 
```

### Sub-Process
```text
- Read `docs/session-plans/121-Session-Plan-Initial.md`
- Create a step-by-step implementation plan complete with production-grade code snippets for how an LLM can teacher the content to a human.
- Break it down into multiple sub-sessions targeting <50k tokens each.
- Take inspiration for the plan/sub-plans strategy from `docs/session-plans/template/`
```

### Summarize
```text
Restructure this session plan as teaching material:

  1. Add a "Teaching Focus" section (3-4 bullet points of concepts learned)
  2. Frame the work with "The Problem" and "The Solution"
  3. Add "Why X?" explanations after major code blocks
  4. Include a "Prerequisites Check" with verification commands
  5. End with "Key Concepts Learned" summary
  6. Add "Next Session" link if applicable

Follow the structure in docs/session-plans/42.1-Session-Plan.md as a template.
```

### Gordon Ramsay
```text
Read:
- `CRITICAL_OPERATING_CONSTRAINTS.md`
- `docs/session-plans/121.3.2-Session-Plan.md`
- `docs/session-plans/121.3.2.1-Session-Plan.md`
- `docs/session-plans/121.3.2.2-Session-Plan.md`
- `docs/session-plans/121.3.2.3-Session-Plan.md`
- `docs/session-plans/121.3.2.4-Session-Plan.md`
- `docs/session-plans/121.3.2.5-Session-Plan.md`
- `docs/session-plans/121.3.2.6-Session-Plan.md`

- Audit these and cross reference the contents against the current state of the project to avoid making assumptions.
- The audit should identify gaps in the plan.
- Raise issues where the repo is missing something that the plan needs/assumes already exists.
- Identify when the plan deviates from patterns that already exist in the repo.

Review the intended implementation quality of ONLY:
- `docs/session-plans/121.3.2.1-Session-Plan.md`
- `docs/session-plans/121.3.2.2-Session-Plan.md`
- `docs/session-plans/121.3.2.3-Session-Plan.md`
- `docs/session-plans/121.3.2.4-Session-Plan.md`
- `docs/session-plans/121.3.2.5-Session-Plan.md`
- `docs/session-plans/121.3.2.6-Session-Plan.md`
```

```text
Read:
- `CRITICAL_OPERATING_CONSTRAINTS.md`
- `docs/session-plans/121.3.2-Session-Plan.md`
- `docs/session-plans/121.3.2.1-Session-Plan.md`
- `docs/session-plans/121.3.2.2-Session-Plan.md`
- `docs/session-plans/121.3.2.3-Session-Plan.md`
- `docs/session-plans/121.3.2.4-Session-Plan.md`
- `docs/session-plans/121.3.2.5-Session-Plan.md`
- `docs/session-plans/121.3.2.6-Session-Plan.md`

Here is a review of the plan:
- `.reviews/20260208-083134.md`
```

### Other
```text
- Read `CRITICAL_OPERATING_CONSTRAINTS.md`
- use the /pm SKILL to audit the `PONE-39` Epic
- Cross reference it's contents against the current state of the project to avoid making poor assumptions.
- Teach me by presenting bite-sized chunks for me to write/edit keeping the commentary separate from the code snippets to make the code snippets easier for me to follow
- **The audit is important, but teaching is the ultimate goal of this session**
```

### Other
```text
Read `CRITICAL_OPERATING_CONSTRAINTS.md`, `docs/session-plans/120.2-Session-Plan.md` and `docs/session-plans/120-Session-Plan.md`.

Please audit `docs/session-plans/120.2-Session-Plan.md` and cross reference it's contents against the current state of the project to avoid making poor assumptions.

I want you to teach me by presenting me with bite-sized chunks for me to write/edit keeping the commentary separate from the code snippets to make the code snippets easier for me to follow

**The audit is important, but teaching is the ultimate goal of this session**
```

### End of Session Sanity Check
```text
Builds clean and all tests pass. Please sanity check that everything in `docs/session-plans/120.2-Session-Plan.md` was implemented as expected.
```

### Update Docs
```text
Please update `docs/session-plans/120.2-Session-Plan.md` and `docs/session-plans/120-Session-Plan.md`
```

### Commit
```text
I have staged all files, please commit without a byline.
```

---
