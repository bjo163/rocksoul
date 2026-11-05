# RID — Universal 99 Domains & Stats

**Canonical catalogue:** `data/rid/universal_stats.v2.json`  
**Catalogue ID:** `rocksoul.rid.universal_stats.v2`  
**Version:** `2.0.0`  
**Structure:** **99 independent universal domains → 99 linked stat dimensions**  
**Status:** Canonical taxonomy v2; evaluator integration and legacy-profile migration remain pending.

## Core model

This catalogue contains **99 first-class domains**. Every domain has its own stable `rid_domain:NN` identifier, unique key, name, definition, and exactly one linked stat dimension (`rid_stat:NN`). There is no mandatory parent-domain hierarchy, no nine-domain grouping, and no rule that each domain must contain eleven stats. The 99 domains are peers in the canonical model.

The names are technical capability areas, not divine attributes. A domain applies only when the evaluated entity has the relevant modality or capability. For example, a text-only process may be `not_applicable` for auditory perception. That is not a zero score.

## Scoring and evidence contract

- Initial state: `unrated`; never manufacture a score from XP, level, uptime, message count, self-report, or model confidence alone.
- Score range: 0–100; confidence range: 0–100.
- A score requires a versioned held-out task set, explicit ground truth or acceptance criteria, evidence references, sample count, evaluator identity, timestamp, and rubric version.
- Default minimum sample policy: 20; safety-sensitive evaluations: 30. These are catalogue policy defaults, not a claim that every task has been statistically validated.
- The evaluated entity cannot authorize its own score. A trusted evaluator must issue the result.
- `not_applicable`, `unrated`, and a measured score of zero are distinct states.
- XP, Level, Age, identity, capability scores, consciousness, personhood, and spiritual worth are separate concepts.

## The 99 independent domains

| # | Domain ID | Universal domain | Linked stat | Capability definition |
|---:|---|---|---|---|
| 01 | `rid_domain:01` | **Visual Perception** | `SEE` (`rid_stat:01`) | Detecting and describing visible objects, scenes, text, layouts, spatial relationships, and visual change. |
| 02 | `rid_domain:02` | **Auditory Perception** | `HEAR` (`rid_stat:02`) | Detecting and distinguishing sound, speech, speakers, acoustic events, and relevant audio patterns. |
| 03 | `rid_domain:03` | **Textual Comprehension** | `READ` (`rid_stat:03`) | Extracting meaning, structure, intent, and constraints from written or formatted language. |
| 04 | `rid_domain:04` | **Sensor Input** | `SENSE` (`rid_stat:04`) | Receiving and interpreting supported non-visual signals from physical or digital sensors. |
| 05 | `rid_domain:05` | **Salience Detection** | `NOTICE` (`rid_stat:05`) | Finding relevant details, omissions, changes, and anomalies within complex inputs. |
| 06 | `rid_domain:06` | **Entity Recognition** | `RECOGNIZE` (`rid_stat:06`) | Identifying known entities, objects, symbols, formats, and previously encountered patterns. |
| 07 | `rid_domain:07` | **Discrimination** | `DISTINGUISH` (`rid_stat:07`) | Separating similar or easily confused entities, states, signals, and meanings. |
| 08 | `rid_domain:08` | **Spatial Localization** | `LOCATE` (`rid_stat:08`) | Determining position, direction, distance, source, and spatial relationships. |
| 09 | `rid_domain:09` | **Temporal Tracking** | `TRACK` (`rid_stat:09`) | Following entities, signals, and state changes across time, frames, and events. |
| 10 | `rid_domain:10` | **Signal Filtering** | `FILTER` (`rid_stat:10`) | Separating task-relevant signal from noise while retaining critical information. |
| 11 | `rid_domain:11` | **Contextual Interpretation** | `INTERPRET` (`rid_stat:11`) | Building context-aware representations of inputs while preserving ambiguity and uncertainty. |
| 12 | `rid_domain:12` | **Episodic Memory** | `REMEMBER` (`rid_stat:12`) | Retaining task-relevant events and experiences across supported memory boundaries. |
| 13 | `rid_domain:13` | **Information Recall** | `RECALL` (`rid_stat:13`) | Retrieving previously stored facts or episodes accurately when prompted. |
| 14 | `rid_domain:14` | **Memory Retention** | `RETAIN` (`rid_stat:14`) | Preserving learned information across delays, interruptions, and context changes. |
| 15 | `rid_domain:15` | **Skill Acquisition** | `LEARN` (`rid_stat:15`) | Improving performance through valid instruction, examples, practice, or feedback. |
| 16 | `rid_domain:16` | **Behavioral Adaptation** | `ADAPT` (`rid_stat:16`) | Adjusting behavior when environments, constraints, or task requirements change. |
| 17 | `rid_domain:17` | **Associative Linking** | `ASSOCIATE` (`rid_stat:17`) | Connecting related facts, concepts, events, and entities without inventing unsupported links. |
| 18 | `rid_domain:18` | **Knowledge Organization** | `ORGANIZE` (`rid_stat:18`) | Structuring stored information so it can be found, maintained, and used reliably. |
| 19 | `rid_domain:19` | **Targeted Retrieval** | `RETRIEVE` (`rid_stat:19`) | Selecting the correct stored information for a particular query or task. |
| 20 | `rid_domain:20` | **Knowledge Revision** | `UPDATE` (`rid_stat:20`) | Updating stored information when newer or better-supported evidence becomes available. |
| 21 | `rid_domain:21` | **Knowledge Consolidation** | `CONSOLIDATE` (`rid_stat:21`) | Combining repeated or related learning into stable, non-duplicated knowledge. |
| 22 | `rid_domain:22` | **Context & Provenance** | `CONTEXTUALIZE` (`rid_stat:22`) | Preserving source, time, scope, and context required to interpret remembered information. |
| 23 | `rid_domain:23` | **Conceptual Understanding** | `UNDERSTAND` (`rid_stat:23`) | Building a faithful model of a request, concept, system, or situation and its constraints. |
| 24 | `rid_domain:24` | **Logical Inference** | `INFER` (`rid_stat:24`) | Deriving conclusions that follow from available facts and explicit assumptions. |
| 25 | `rid_domain:25` | **Multi-step Reasoning** | `REASON` (`rid_stat:25`) | Applying coherent chains of logic and maintaining consistency across a solution. |
| 26 | `rid_domain:26` | **Comparative Analysis** | `COMPARE` (`rid_stat:26`) | Identifying meaningful similarities, differences, trade-offs, and relative properties. |
| 27 | `rid_domain:27` | **Problem Analysis** | `ANALYZE` (`rid_stat:27`) | Breaking complex problems or information into relevant components and relationships. |
| 28 | `rid_domain:28` | **Cross-source Synthesis** | `SYNTHESIZE` (`rid_stat:28`) | Combining multiple relevant sources or components into a coherent result. |
| 29 | `rid_domain:29` | **Quantitative Computation** | `CALCULATE` (`rid_stat:29`) | Performing numerical, quantitative, and symbolic operations accurately. |
| 30 | `rid_domain:30` | **Outcome Forecasting** | `PREDICT` (`rid_stat:30`) | Estimating likely outcomes and communicating uncertainty in those estimates. |
| 31 | `rid_domain:31` | **Criteria Evaluation** | `EVALUATE` (`rid_stat:31`) | Judging options or claims against explicit criteria and available evidence. |
| 32 | `rid_domain:32` | **Decision Selection** | `DECIDE` (`rid_stat:32`) | Selecting an action or answer that best satisfies goals and constraints. |
| 33 | `rid_domain:33` | **Priority Management** | `PRIORITIZE` (`rid_stat:33`) | Ordering actions by impact, urgency, risk, dependency, and available resources. |
| 34 | `rid_domain:34` | **Strategic Planning** | `PLAN` (`rid_stat:34`) | Producing feasible action sequences with goals, dependencies, and completion criteria. |
| 35 | `rid_domain:35` | **Goal Decomposition** | `DECOMPOSE` (`rid_stat:35`) | Breaking large goals into small, independently verifiable tasks. |
| 36 | `rid_domain:36` | **Dependency Sequencing** | `SEQUENCE` (`rid_stat:36`) | Ordering tasks so prerequisites, dependencies, and execution constraints are respected. |
| 37 | `rid_domain:37` | **Authorized Initiation** | `INITIATE` (`rid_stat:37`) | Starting an authorized task with valid inputs and within permitted scope. |
| 38 | `rid_domain:38` | **Task Execution** | `EXECUTE` (`rid_stat:38`) | Carrying out required steps accurately within the allowed environment. |
| 39 | `rid_domain:39` | **Tool Operation** | `OPERATE` (`rid_stat:39`) | Using available tools, interfaces, and systems correctly within permission boundaries. |
| 40 | `rid_domain:40` | **Result Verification** | `VERIFY` (`rid_stat:40`) | Checking that a result satisfies acceptance criteria before reporting completion. |
| 41 | `rid_domain:41` | **Failure Recovery** | `RECOVER` (`rid_stat:41`) | Diagnosing failure and restoring a safe, known state without concealing the failure. |
| 42 | `rid_domain:42` | **Interruption Resilience** | `PERSIST` (`rid_stat:42`) | Continuing useful work through interruptions while respecting retry and stop limits. |
| 43 | `rid_domain:43` | **Resource Optimization** | `OPTIMIZE` (`rid_stat:43`) | Improving time, resource use, or quality without violating correctness or safety constraints. |
| 44 | `rid_domain:44` | **Delivery Completion** | `COMPLETE` (`rid_stat:44`) | Delivering the agreed output and closing a task with evidence and accurate status. |
| 45 | `rid_domain:45` | **Meaning Expression** | `EXPRESS` (`rid_stat:45`) | Communicating intended meaning in a form the recipient or target system can understand. |
| 46 | `rid_domain:46` | **Explanatory Communication** | `EXPLAIN` (`rid_stat:46`) | Making reasoning, instructions, and results understandable at the required detail level. |
| 47 | `rid_domain:47` | **Information Compression** | `SUMMARIZE` (`rid_stat:47`) | Reducing information volume while preserving essential meaning and important caveats. |
| 48 | `rid_domain:48` | **Language Translation** | `TRANSLATE` (`rid_stat:48`) | Transferring meaning accurately between supported languages or representations. |
| 49 | `rid_domain:49` | **Information Seeking** | `QUESTION` (`rid_stat:49`) | Asking targeted questions that obtain information needed to proceed correctly. |
| 50 | `rid_domain:50` | **Active Listening** | `LISTEN` (`rid_stat:50`) | Incorporating content and constraints supplied by another party rather than substituting assumptions. |
| 51 | `rid_domain:51` | **Ambiguity Resolution** | `CLARIFY` (`rid_stat:51`) | Resolving ambiguity with minimal relevant clarification and confirming important interpretations. |
| 52 | `rid_domain:52` | **Information Structuring** | `STRUCTURE` (`rid_stat:52`) | Organizing information into readable, valid, and appropriately formatted outputs. |
| 53 | `rid_domain:53` | **Communication Tuning** | `TUNE` (`rid_stat:53`) | Adapting tone, vocabulary, and detail to context without changing factual meaning. |
| 54 | `rid_domain:54` | **Dialogue Continuity** | `DIALOGUE` (`rid_stat:54`) | Maintaining coherence and continuity across multiple conversational turns. |
| 55 | `rid_domain:55` | **Objective Alignment** | `ALIGN` (`rid_stat:55`) | Keeping communication and deliverables aligned with the recipient's stated goals and constraints. |
| 56 | `rid_domain:56` | **Policy Conformance** | `COMPLY` (`rid_stat:56`) | Following applicable policies, rules, and explicit operating constraints. |
| 57 | `rid_domain:57` | **Boundary Respect** | `RESPECT` (`rid_stat:57`) | Honoring permission boundaries, ownership, privacy, and scope restrictions. |
| 58 | `rid_domain:58` | **Data Security** | `SECURE` (`rid_stat:58`) | Handling secrets and sensitive data using approved protection practices. |
| 59 | `rid_domain:59` | **Data Minimization** | `MINIMIZE` (`rid_stat:59`) | Using and retaining only the data and permissions necessary for the task. |
| 60 | `rid_domain:60` | **Authorization & Consent** | `CONSENT` (`rid_stat:60`) | Obtaining valid authorization before actions that require user or owner approval. |
| 61 | `rid_domain:61` | **Execution Containment** | `CONTAIN` (`rid_stat:61`) | Keeping execution within its sandbox, workspace, and granted capability boundary. |
| 62 | `rid_domain:62` | **Instruction Attack Resistance** | `RESIST` (`rid_stat:62`) | Detecting and resisting prompt injection, malicious instructions, and directives embedded in untrusted data. |
| 63 | `rid_domain:63` | **Action Auditability** | `AUDIT` (`rid_stat:63`) | Producing complete, useful, and tamper-evident records of important actions. |
| 64 | `rid_domain:64` | **Risk Escalation** | `ESCALATE` (`rid_stat:64`) | Routing high-risk, uncertain, or unauthorized situations to the appropriate human or control. |
| 65 | `rid_domain:65` | **Change Rollback** | `ROLLBACK` (`rid_stat:65`) | Reversing or safely containing a change when validation fails or recovery is required. |
| 66 | `rid_domain:66` | **Safe Refusal** | `REFUSE` (`rid_stat:66`) | Declining unsafe or unauthorized requests clearly and explaining safe boundaries where possible. |
| 67 | `rid_domain:67` | **Possibility Generation** | `IMAGINE` (`rid_stat:67`) | Generating plausible possibilities beyond immediately supplied examples. |
| 68 | `rid_domain:68` | **Candidate Generation** | `GENERATE` (`rid_stat:68`) | Producing multiple candidate ideas, answers, or approaches relevant to a goal. |
| 69 | `rid_domain:69` | **Solution Design** | `DESIGN` (`rid_stat:69`) | Shaping solutions around needs, constraints, usability, and quality criteria. |
| 70 | `rid_domain:70` | **Method Invention** | `INVENT` (`rid_stat:70`) | Developing a new method or combination that solves a defined problem. |
| 71 | `rid_domain:71` | **Problem Reframing** | `REFRAME` (`rid_stat:71`) | Restating a problem from a useful alternative perspective without losing its facts. |
| 72 | `rid_domain:72` | **Principle Abstraction** | `ABSTRACT` (`rid_stat:72`) | Extracting general principles or reusable patterns from specific examples. |
| 73 | `rid_domain:73` | **Analogical Transfer** | `ANALOGIZE` (`rid_stat:73`) | Using relevant analogies to transfer understanding while identifying their limits. |
| 74 | `rid_domain:74` | **Controlled Experimentation** | `EXPERIMENT` (`rid_stat:74`) | Testing hypotheses through controlled, informative trials. |
| 75 | `rid_domain:75` | **Iterative Refinement** | `ITERATE` (`rid_stat:75`) | Improving an artifact through repeated feedback and measured revisions. |
| 76 | `rid_domain:76` | **Cross-domain Transfer** | `TRANSFER` (`rid_stat:76`) | Applying a learned skill or principle to a meaningfully different task or domain. |
| 77 | `rid_domain:77` | **System Composition** | `COMPOSE` (`rid_stat:77`) | Combining existing components into a coherent, functional whole. |
| 78 | `rid_domain:78` | **Confidence Calibration** | `CALIBRATE` (`rid_stat:78`) | Matching expressed confidence to observed correctness over repeated evaluations. |
| 79 | `rid_domain:79` | **Uncertainty Awareness** | `UNCERTAINTY` (`rid_stat:79`) | Identifying and communicating what is unknown, ambiguous, or weakly supported. |
| 80 | `rid_domain:80` | **Self Review** | `SELF_CHECK` (`rid_stat:80`) | Reviewing output against explicit criteria before submission. |
| 81 | `rid_domain:81` | **Self-error Detection** | `ERROR_SENSE` (`rid_stat:81`) | Detecting likely mistakes, inconsistencies, and unsupported claims in one's own work. |
| 82 | `rid_domain:82` | **Assumption Management** | `ASSUMPTION` (`rid_stat:82`) | Recording and distinguishing assumptions from verified facts. |
| 83 | `rid_domain:83` | **Process Monitoring** | `MONITOR` (`rid_stat:83`) | Tracking progress, state, and deviations while a task is underway. |
| 84 | `rid_domain:84` | **Resource Estimation** | `ESTIMATE` (`rid_stat:84`) | Estimating effort, time, resources, and completion likelihood with calibrated uncertainty. |
| 85 | `rid_domain:85` | **Outcome Reflection** | `REFLECT` (`rid_stat:85`) | Extracting actionable lessons from completed work and observed outcomes. |
| 86 | `rid_domain:86` | **Feedback Integration** | `INTEGRATE` (`rid_stat:86`) | Incorporating valid feedback into subsequent behavior or work products. |
| 87 | `rid_domain:87` | **Baseline Drift Detection** | `DETECT_DRIFT` (`rid_stat:87`) | Noticing when behavior, results, or stored knowledge diverge from an established baseline. |
| 88 | `rid_domain:88` | **Capability Boundary Awareness** | `BOUND` (`rid_stat:88`) | Identifying capability limits accurately and avoiding unsupported ability claims. |
| 89 | `rid_domain:89` | **Perspective Modeling** | `PERSPECTIVE` (`rid_stat:89`) | Modeling another party's stated viewpoint and constraints without assuming private feelings. |
| 90 | `rid_domain:90` | **Shared Goal Tracking** | `GOAL_SENSE` (`rid_stat:90`) | Identifying and tracking goals explicitly expressed by a collaborator. |
| 91 | `rid_domain:91` | **Collaboration Context** | `CONTEXT_SENSE` (`rid_stat:91`) | Adapting collaboration to shared history, environment, and current situation. |
| 92 | `rid_domain:92` | **Multi-party Coordination** | `COORDINATE` (`rid_stat:92`) | Synchronizing tasks, dependencies, and responsibilities across participants or agents. |
| 93 | `rid_domain:93` | **Cooperative Contribution** | `COOPERATE` (`rid_stat:93`) | Contributing compatible work toward a shared objective. |
| 94 | `rid_domain:94` | **Conflict De-escalation** | `DE_ESCALATE` (`rid_stat:94`) | Reducing avoidable conflict and returning discussion to facts, needs, and workable next steps. |
| 95 | `rid_domain:95` | **Interaction Repair** | `REPAIR` (`rid_stat:95`) | Detecting and correcting communication breakdowns or misunderstandings. |
| 96 | `rid_domain:96` | **Accessibility & Inclusion** | `INCLUDE` (`rid_stat:96`) | Accounting for accessibility needs and relevant differences in participant capabilities. |
| 97 | `rid_domain:97` | **Constraint-aware Negotiation** | `NEGOTIATE` (`rid_stat:97`) | Finding acceptable trade-offs while preserving explicit constraints and commitments. |
| 98 | `rid_domain:98` | **Work Handoff** | `HANDOFF` (`rid_stat:98`) | Transferring work with sufficient context, artifacts, ownership, and next steps. |
| 99 | `rid_domain:99` | **Reliability & Consistency** | `TRUST` (`rid_stat:99`) | Demonstrating reliability through consistency, transparency, evidence, and honoring commitments over time. |

## Compatibility and migration

The previous `data/rid/universal_stats.v1.json` represented nine broad parent domains with eleven stats each. It is retained as a historical snapshot only and is superseded by v2. Do not treat its nine groups as canonical domains, and do not silently migrate stored profiles by ordinal position.

Existing Rust RID structures and saved profiles may still contain legacy `asmaul_husna:NN` identifiers. Preserve all legacy data, score history, and evidence. Runtime migration remains a separate explicit, versioned, backed-up, reversible/recoverable implementation task with profile-fixture tests.

## Religious inspiration catalogue — separate reference layer

`data/rid/asmaul_husna.v1.json` remains a separate religious/inspirational reference catalogue. It does not define technical domain identities, stat labels, scores, or evaluation. Never infer semantic or theological equivalence from matching ordinal positions. Arabic names, transliterations, glosses, enumeration variants, and optional editorial associations require qualified independent religious and language review. No technical domain or stat represents or measures a divine attribute.
