# Simulation Mathematics, Physical Laws and Domain Model Specifications

> **Document ID:** LWE-SIM-001 | **Status:** proposed quantitative foundations. Examples are deliberately simplified; model validity requires calibration, testing and domain review.

## Simulation kernel

Use a **hybrid discrete-event / adaptive-time-step** scheduler. Domain modules specify state vector, units, numeric integrator or transition reducer, allowed time-step range, truncation/error tolerance, coupling inputs/outputs and conservation contracts. A scheduler chooses the next discrete event or integration barrier. Multiple rates are allowed only where exchanged data are synchronised and stale values are bounded. Domains must not “peek into the future” of another domain.

```text
run_until(target_time):
  while permitted_work_exists:
    barrier := min(next_discrete_event, next_model_required_boundary, target_time)
    for each due continuous domain in topological/coupling order:
       advance_with_tolerance(domain, barrier, pinned_parameter_version, rng_stream)
    for each eligible discrete event at barrier:
       validate_proposed_transition(event)
       atomic_commit_or_reject(event)
    update projections, causal indexes, known uncertainty, checkpoints
```

Real temporal loops need separate constraint-solver semantics, not a naïve infinite scheduling cycle. Loosely coupled domain interfaces may use monotonic barriers and bounded iterative convergence rather than unsafe ad hoc ordering.

## General invariants and dimensional analysis

Use tagged units and strict commodity identities. Accounting identities are exact integers where possible; physical models use error-bounded floating point or fixed-point numeric types. Example resource conservation: `stock_after = stock_before + production + imports − consumption − exports − spoilage` (with authorised sinks/sources). Require `stock_after >= 0` for an inventory that cannot be negative. Model energy should obey a reality-specific transfer/production ledger, including explicit magical potential categories and allowed conversion constraints; do not impose real physics unchanged on fictional planes without an approved law contract.

## Economy model — town prototype

**Entities:** households, producers, firms, workplaces, market stalls, transport agents, bank/credit organisations, treasury, warehouses and resource accounts. **Flows:** wages, production, consumption, taxes, rent, credit, trade, spoilage, gifts/theft and legal confiscation. **Stocks:** grain, coin, timber, livestock, transport capacity, land rights, equipment, credit obligations and food reserves.

A minimal per-tick market model:

- Available grain `S_t = stocks_t + harvest_t + imports_t − committed_exports_t − reserve_locked_t`.
- Desired demand `D_t = Σ_h household_required_food_h + industrial_demand_t + speculative_orders_t`.
- Price adjustment candidate `P_{t+1} = max(P_min, P_t × exp(η × (D_t − S_t)/max(S_t, ε)))`. This is **one candidate synthetic price adjustment**, not a validated real-economy law; cap rate of adjustment and compare with auction/order-book approaches.
- Household budget constraint `consumption_spending <= available_cash + authorised_credit`.
- Transport costs use distance, terrain, weather, labour, risk, spoilage and legal fees.
- Finance requires explicit double-entry or auditable transaction ledgers; currencies are commodities with issuers and issuance events; debts are obligations, not negative commodity stock unless the account contract permits credit.

**Evaluation:** mass balance, no double-spend, stable-price cases, scarcity response, pathological oscillation, distribution and response to shock, invariant preservation under independent seeded stream reordering. Do not call an output “realistic” until calibrated and stress-tested against intended analogues.

## Households, demography and psychology

At town resolution, persist approximately 250 residents and their household links. Birth/death/migration and occupation changes are events. Population identity equation: `N_{t+1} = N_t + births − deaths + in_migrants − out_migrants`; define citizenship vs resident status. Age and health influence labour; beliefs influence economic choices. Individual agents use psychological state variables and constraints rather than universally rational max-utility behaviour. For distant populations, aggregate cohorts with uncertainty and reconciliation rules for when individuals become instantiated.

## Ecology, resources and geography

Use species/population state by habitat and time. Baseline candidate dynamics: `dN_i/dt = births_i − deaths_i + immigration_i − emigration_i`, with food resource, predation, disease and seasonality as covariates; logistic or Lotka–Volterra-like equations are pedagogical starting points only. Landcover, watershed, soil nutrients, erosion, precipitation, runoff and pollutants couple to agriculture/transport/health. Planetary geological and evolutionary evolution require multiscale models and sparse milestone simulation, not individual-organism tracking for billions of years. Geological and astronomical realism should be independently evaluated before claims.

## Biology/medicine

Actor health stores bounded physiologic state (nutrition, hydration, fatigue, immune status, injury records) plus species-specific parameters. A wound includes location, type, severity, blood loss risk, infection probability, treatment and recovery. Disease has transmission model, incubation/infectiousness, susceptibility, within-host abstraction and environmental survivability; outbreaks use ordinary stochastic population models with explicit assumptions. D&D hit points remain a gameplay abstraction over survivability/agency; no automatic translation “18 HP equals 18 grams of tissue.” Magical healing must specify which state variables it actually changes and at what cost. Experimental life science should be well-scoped to fictional mechanisms, not presented as medical advice.

## Magic, metaphysics and divine systems

Define **law contract** per reality for an abstract magical state `M` with conserved or budgeted quantities as chosen. A spell is `preconditions + resource budget + transformation + postconditions + side effects`. A magic system may include transfer or conversion between domains (`ΔE_thermal = f(ΔM, field, location, efficiency)`) with dimensional units, no claim that fictional M is physically established. Different traditions may interpret the same deeper law inaccurately. Gods are special entities with distinct provenance; power does not automatically imply moral goodness, infallibility or unlimited knowledge. Life, death, consciousness, identity and resurrection remain deferred metaphysical definitions requiring separate domain ADR and tests.

## Warfare and institutions

Unit-level simulation models manpower, morale, training, command latency, supply, weather, terrain, magical capabilities and information. The map/3D combat adapter sends validated encounters to D&D rules for tactical play; large war outcomes use aggregate models with uncertainty. Organisations use charter, members, treasury, capabilities, policies, knowledge and strategic goals. Diplomatic acts, laws, mobilisation and treaties are events with downstream dependencies. No faction has a scripted victory.

## Technological emergence

Do not advance along a universal tech tree. Represent discoveries as prerequisite knowledge, equipment, raw materials, experiment histories, institutions, infrastructure and diffusion. Innovations must be physically/magically possible and socially adopted. A society could be advanced in biology but limited in astronomy; technology can regress through destruction or deliberate suppression. Cross-reality technology transfer is subject to law compatibility and actual contact mechanisms.

## Fidelity policy

Each module publishes `resolution_level`, `known_error_bounds`, `statistical_assumptions`, `unmodeled_effects`, `validation_evidence`, `compute_cost_per_tick` and `promotion/demotion rules`. On downgrade to aggregate resolution, preserve accounts, identities, facts and conservation totals. On expansion, generate only details consistent with known data and label inferred information; do not claim latent individual history was explicitly simulated.
