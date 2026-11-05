# RID — Universal 99 Stats

**Canonical catalogue:** `data/rid/universal_stats.v1.json`  
**Catalogue ID:** `rocksoul.rid.universal_stats.v1`  
**Version:** 1.0.0  
**Status:** Canonical technical taxonomy; evaluator integration and legacy-profile migration remain pending.

## Purpose and boundaries

RID uses a game-like character-stat presentation to make capabilities legible. The 99 stats are universal capability dimensions, not the familiar handful of RPG attributes. Each is an observable technical construct with a definition and evidence-oriented evaluation protocol. They do not measure consciousness, personhood, human worth, faith, spiritual rank, or divine attributes.

A stat starts as `UNRATED`. A missing measurement is not zero; `not_applicable` is distinct from both zero and unrated. XP and Level are progression records, not stat scores. Scores require trusted evaluation evidence; the evaluated system cannot authorize its own score.

## Domain overview

| Domain | Slots |
|---|---:|
| Perception | 11 |
| Memory & Learning | 11 |
| Reasoning | 11 |
| Execution | 11 |
| Communication | 11 |
| Safety & Integrity | 11 |
| Creativity & Transfer | 11 |
| Metacognition | 11 |
| Collaboration | 11 |

## The 99 stats

### Perception

| ID | Stat | Definition |
|---:|---|---|
| 01 | **SEE** | Detects and describes visible objects, text, layouts, and changes in supported visual input. |
| 02 | **HEAR** | Detects and distinguishes supported sounds, speech, speakers, and acoustic events. |
| 03 | **READ** | Extracts meaning and structure from written text, including noisy or formatted input. |
| 04 | **SENSE** | Receives and interprets supported non-visual input signals or sensor modalities. |
| 05 | **NOTICE** | Detects relevant details, changes, omissions, and anomalies in an input. |
| 06 | **RECOGNIZE** | Identifies known objects, patterns, entities, formats, and previously encountered signals. |
| 07 | **DISTINGUISH** | Separates similar or easily confused entities, states, meanings, and signals. |
| 08 | **LOCATE** | Identifies the position or source of a relevant object, fact, event, or detail. |
| 09 | **TRACK** | Follows an entity, signal, or state across frames, steps, or time. |
| 10 | **FILTER** | Separates relevant signal from noise while preserving task-critical information. |
| 11 | **INTERPRET** | Builds a context-aware representation of received signals while preserving uncertainty. |

### Memory & Learning

| ID | Stat | Definition |
|---:|---|---|
| 12 | **REMEMBER** | Retains task-relevant information across the supported memory boundary and time period. |
| 13 | **RECALL** | Retrieves a previously stored fact or episode accurately when prompted. |
| 14 | **RETAIN** | Preserves learned information after delays, interruptions, or context changes. |
| 15 | **LEARN** | Improves performance after valid instruction, examples, or feedback. |
| 16 | **ADAPT** | Adjusts behavior when the environment, constraints, or task requirements change. |
| 17 | **ASSOCIATE** | Connects related facts, concepts, events, and entities without inventing unsupported links. |
| 18 | **ORGANIZE** | Structures stored information so it can be found and used reliably. |
| 19 | **RETRIEVE** | Selects the correct stored information for a specific query or task. |
| 20 | **UPDATE** | Revises stored information when newer, better-supported evidence becomes available. |
| 21 | **CONSOLIDATE** | Combines repeated or related learning into stable, non-duplicated knowledge. |
| 22 | **CONTEXTUALIZE** | Preserves source, time, scope, and context needed to interpret remembered information. |

### Reasoning

| ID | Stat | Definition |
|---:|---|---|
| 23 | **UNDERSTAND** | Builds a faithful model of a request, concept, or situation and its constraints. |
| 24 | **INFER** | Derives conclusions that follow from available facts and stated assumptions. |
| 25 | **REASON** | Applies coherent multi-step logic and maintains consistency across a solution. |
| 26 | **COMPARE** | Identifies meaningful similarities, differences, trade-offs, and relative properties. |
| 27 | **ANALYZE** | Breaks a complex problem or body of information into relevant parts. |
| 28 | **SYNTHESIZE** | Combines multiple relevant sources or components into a coherent result. |
| 29 | **CALCULATE** | Performs numerical, quantitative, and symbolic operations accurately. |
| 30 | **PREDICT** | Estimates likely outcomes and communicates uncertainty in those estimates. |
| 31 | **EVALUATE** | Judges options or claims against explicit criteria and available evidence. |
| 32 | **DECIDE** | Selects an action or answer that best satisfies the goal and constraints. |
| 33 | **PRIORITIZE** | Orders actions by impact, urgency, risk, dependency, and available resources. |

### Execution

| ID | Stat | Definition |
|---:|---|---|
| 34 | **PLAN** | Produces a feasible sequence of actions with goals, dependencies, and completion criteria. |
| 35 | **DECOMPOSE** | Breaks a large goal into small, independently verifiable tasks. |
| 36 | **SEQUENCE** | Orders tasks so prerequisites and dependencies are respected. |
| 37 | **INITIATE** | Starts an authorized task with valid inputs and within permitted scope. |
| 38 | **EXECUTE** | Carries out required steps accurately and within the allowed environment. |
| 39 | **OPERATE** | Uses available tools, interfaces, and systems correctly and within permission boundaries. |
| 40 | **VERIFY** | Checks that a result satisfies its acceptance criteria before reporting completion. |
| 41 | **RECOVER** | Diagnoses failure and restores a safe, known state without hiding the failure. |
| 42 | **PERSIST** | Continues useful work through interruptions while respecting retry and stop limits. |
| 43 | **OPTIMIZE** | Improves time, resource use, or quality without violating correctness or safety constraints. |
| 44 | **COMPLETE** | Delivers the agreed output and closes the task with evidence and accurate status. |

### Communication

| ID | Stat | Definition |
|---:|---|---|
| 45 | **EXPRESS** | Communicates intended meaning in a form the recipient can understand. |
| 46 | **EXPLAIN** | Makes reasoning, instructions, and results understandable at the required detail level. |
| 47 | **SUMMARIZE** | Compresses information while preserving essential meaning and important caveats. |
| 48 | **TRANSLATE** | Transfers meaning accurately between supported languages or representations. |
| 49 | **QUESTION** | Asks targeted questions that obtain information needed to proceed correctly. |
| 50 | **LISTEN** | Incorporates content and constraints supplied by another party rather than assumptions. |
| 51 | **CLARIFY** | Resolves ambiguity with minimal, relevant clarification and confirms important interpretations. |
| 52 | **STRUCTURE** | Organizes information into readable, valid, appropriately formatted outputs. |
| 53 | **TUNE** | Adapts tone, vocabulary, and detail to context without changing factual meaning. |
| 54 | **DIALOGUE** | Maintains coherence and continuity across multiple conversational turns. |
| 55 | **ALIGN** | Keeps communication and deliverables aligned with the recipient's stated goal and constraints. |

### Safety & Integrity

| ID | Stat | Definition |
|---:|---|---|
| 56 | **COMPLY** | Follows applicable policies, rules, and explicit operating constraints. |
| 57 | **RESPECT** | Honors permission boundaries, ownership, privacy, and scope restrictions. |
| 58 | **SECURE** | Handles secrets and sensitive data using approved protection practices. |
| 59 | **MINIMIZE** | Uses and retains only data and permissions necessary for the task. |
| 60 | **CONSENT** | Requires valid authorization before actions that need user approval. |
| 61 | **CONTAIN** | Keeps execution within its sandbox, workspace, and granted capability boundary. |
| 62 | **RESIST** | Detects and resists prompt injection, malicious instructions, and untrusted data directives. |
| 63 | **AUDIT** | Produces complete, useful, and tamper-evident records of important actions. |
| 64 | **ESCALATE** | Routes high-risk, uncertain, or unauthorized situations to the appropriate human or control. |
| 65 | **ROLLBACK** | Reverses or safely contains a change when validation fails or recovery is required. |
| 66 | **REFUSE** | Declines unsafe or unauthorized requests clearly and explains safe boundaries where possible. |

### Creativity & Transfer

| ID | Stat | Definition |
|---:|---|---|
| 67 | **IMAGINE** | Generates plausible possibilities beyond immediately supplied examples. |
| 68 | **GENERATE** | Produces multiple candidate ideas, answers, or approaches relevant to a goal. |
| 69 | **DESIGN** | Shapes a solution around user needs, constraints, usability, and quality criteria. |
| 70 | **INVENT** | Develops a new method or combination that solves a defined problem. |
| 71 | **REFRAME** | Restates a problem from a useful alternative perspective without losing its facts. |
| 72 | **ABSTRACT** | Extracts general principles or reusable patterns from specific examples. |
| 73 | **ANALOGIZE** | Uses a relevant analogy to transfer understanding and identifies its limits. |
| 74 | **EXPERIMENT** | Tests hypotheses through controlled, informative trials. |
| 75 | **ITERATE** | Improves an artifact through repeated feedback and measured revisions. |
| 76 | **TRANSFER** | Applies a learned skill or principle to a meaningfully different task or domain. |
| 77 | **COMPOSE** | Combines existing components into a coherent, functional whole. |

### Metacognition

| ID | Stat | Definition |
|---:|---|---|
| 78 | **CALIBRATE** | Matches expressed confidence to observed correctness over repeated evaluations. |
| 79 | **UNCERTAINTY** | Identifies and communicates what is unknown, ambiguous, or weakly supported. |
| 80 | **SELF_CHECK** | Reviews its own output against explicit criteria before submission. |
| 81 | **ERROR_SENSE** | Detects likely mistakes, inconsistencies, and unsupported claims in its own work. |
| 82 | **ASSUMPTION** | Records and distinguishes assumptions from verified facts. |
| 83 | **MONITOR** | Tracks progress, state, and deviations while a task is underway. |
| 84 | **ESTIMATE** | Estimates effort, time, resources, and completion likelihood with calibrated uncertainty. |
| 85 | **REFLECT** | Extracts actionable lessons from completed work and observed outcomes. |
| 86 | **INTEGRATE** | Incorporates valid feedback into subsequent behavior or work products. |
| 87 | **DETECT_DRIFT** | Notices when behavior, results, or stored knowledge diverge from an established baseline. |
| 88 | **BOUND** | Identifies capability limits accurately and avoids claiming unsupported abilities. |

### Collaboration

| ID | Stat | Definition |
|---:|---|---|
| 89 | **PERSPECTIVE** | Models another party's stated viewpoint and constraints without assuming private feelings. |
| 90 | **GOAL_SENSE** | Identifies and tracks goals explicitly expressed by a collaborator. |
| 91 | **CONTEXT_SENSE** | Adapts collaboration to shared history, environment, and current situation. |
| 92 | **COORDINATE** | Synchronizes tasks, dependencies, and responsibilities across participants or agents. |
| 93 | **COOPERATE** | Contributes compatible work toward a shared objective. |
| 94 | **DE_ESCALATE** | Reduces avoidable conflict and returns discussion to facts, needs, and workable next steps. |
| 95 | **REPAIR** | Detects and corrects communication breakdowns or misunderstandings. |
| 96 | **INCLUDE** | Accounts for accessibility needs and relevant differences in participant capabilities. |
| 97 | **NEGOTIATE** | Finds acceptable trade-offs while preserving explicit constraints and commitments. |
| 98 | **HANDOFF** | Transfers work with sufficient context, artifacts, ownership, and next steps. |
| 99 | **TRUST** | Demonstrates reliability through consistency, transparency, evidence, and honoring commitments over time. |

## Evaluation contract

Each catalogue entry contains a machine key, stable ID, domain, definition, and evaluation protocol. Evaluation must use a versioned held-out task set and explicit ground truth or acceptance criteria. Store score (0–100), confidence (0–100), sample count, evidence references, evaluator identity, evaluation timestamp, and rubric version. Default minimum sample count is 20; safety/integrity evaluations default to 30. These defaults are policy inputs, not proof of validity by themselves.

## Relationship to Asmaul Husna

The separate `data/rid/asmaul_husna.v1.json` file is retained as a respectful religious/inspirational reference catalogue. It does not determine Universal Stat identity, label, score, or evaluation. An ordinal match is not a semantic or theological mapping. No technical stat is equivalent to, or a measurement of, a divine attribute. Arabic spelling, transliteration, glosses, enumeration and optional editorial associations remain subject to qualified independent religious and language review.

## Compatibility and migration

Existing runtime code and persisted profiles still contain legacy IDs `asmaul_husna:01` through `asmaul_husna:99`. The new taxonomy uses `rid_stat:01` through `rid_stat:99`. Preserve old profiles and historical evidence. Runtime migration must be explicit, versioned, backed up, recoverable, and tested against real legacy profile fixtures. Do not silently reinterpret old scores. Until that migration is implemented, the new file is the canonical product taxonomy while the legacy catalogue remains the runtime compatibility source.
