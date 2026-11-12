# ADR-0003 — Merging replays interventions, not simulations
Status: Accepted provisionally under Decision 61
Date: 2026-11-12
Applicable decisions: 01, 22, 27
Context/problem: Two branches that each simulated the same fictional days cannot be combined by concatenating their ticks. The specification asks for conservative reconciliation that never rewrites a source and never invents a fix.
Decision and rationale: A merge takes the source's explicit world interventions since the lowest common ancestor (sales, destruction, issued resources, recorded and protected facts, weather shocks, policy changes, planted grievances, NPC edits, openings of magical sources) plus deaths that play caused, orders them by fictional day, and applies each through the ordinary reducer on a copy of the target's head — first advancing the target if the event lies in its future. The first precondition failure is a conflict, reported with the violated rule and the target events that touched the same entities. Approval is owner-only, bound to a digest of both heads and the event set, and creates a **new** branch.
What this deliberately does not do: carry over conversations, reputations, thefts or the emergent social history of a campaign; guarantee that all semantic conflicts are found (two facts that contradict in prose will both merge). False conflicts are preferred to silent damage.
Proof: `at004_…blocks_the_merge`, `at005_007_…non_sibling_ancestry`, `deaths_caused_in_play_merge_into_canon`.
