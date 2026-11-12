# IP Rights, Licensing, Copyright, Community and Marketplace Policy

> **Document ID:** LWE-IP-001 | **Approved basis:** 05, 12, 30, 38–41 | **Status:** product policy proposal requiring qualified legal review before commercial launch.

## Separate rights layers

1. **Engine code:** governed by a chosen software licence; not selected by Decision 39.
2. **Game rules:** 2014 SRD 5.1 and 2024 SRD 5.2.1 content available under CC BY 4.0 with required attribution; protected D&D terms and content outside SRD require separate review.
3. **Original world lore:** author-defined copyright/licence subject to jurisdiction and creation provenance.
4. **Generated images/voices/music:** provider terms, input/reference rights, voice/likeness consent and jurisdiction-specific originality requirements.
5. **Community mods:** software licence and capability manifest independent of content rights.
6. **Research corpus:** bibliographic metadata, limited quotations or rights-cleared full texts; no assumption that a book may be mirrored or used for training.
7. **Collaboration:** explicit contribution ownership/assignment/licensing, attribution and revocation/export terms.

## Rights manifest format (illustrative)

```yaml
asset_id: town_market_painting_001
asset_kind: illustration
creator_principal: creator_023
provenance:
  type: human_and_model_assisted
  model_provider: provider_name
  model_version: declared_version
  source_refs: [licensed_reference_001]
rights:
  license_id: custom_private_v1
  redistribution: false
  derivatives: owner_permission_required
  commercial_use: owner_permission_required
  attribution_text: "..."
consent:
  identifiable_real_person: false
  voice_clone_permission_ref: null
review:
  license_scan_status: pending
  approved_by: null
```

**Not legal advice:** author ownership of AI outputs and enforceability of restrictions may vary. Tooling must record claims and permissions, not certify rights as a matter of law.

## Mod packaging and publishing

Package manifest: publisher ID, semantic version, target engine API range, hashes/signature, declared capabilities, dependencies, content licences, creator attribution, safety rating and import migrations. Mods are installed disabled until checks pass. Code modules execute in a sandbox or isolated process with explicit capabilities and CPU/memory/file/network quotas. A mod cannot change canon automatically; owner must approve data schema/contract migration and policy elevation.

## Writers' Brains use of living/legacy authors

Use public research to identify documented craft principles; cite sources. Do not represent outputs as written or endorsed by the actual authors. Do not reproduce lengthy passages of copyrighted books or train a derivative model on works without appropriate rights. Avoid style imitation framed as a substitute for a specific living author's current work. The council should express its own independent recommendations grounded in evidence and transparent attribution.

## Community commercial model

Marketplace payments, disputes, royalties, taxes, refunds, sanctions and international consumer laws are **separate future workstreams**. Do not pick a payments platform or promise revenue-sharing percentages without an ADR and commercial validation. A creator may share a world under a different licence than the engine; user experience must display effective permitted actions and export limitations.

## Release checklist

Prior to any paid or public distribution: formal legal review of source components and SRD attribution, audit of third-party assets, source/consent provenance, content moderation scope, terms of service and privacy notice, contributor licence agreement approach, marketplace tax/reporting obligations and repeat infringement response policy. Keep an auditable record of rights decisions.
