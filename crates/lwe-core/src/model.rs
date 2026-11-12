//! Materialised world state. Everything here is a *projection* of the event log:
//! it is rebuilt by replaying events and must never be mutated except through
//! `reducer::apply`. All maps are `BTreeMap` so serialisation (and therefore the
//! state hash) is canonical.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Id = String;
/// One whole unit of any commodity is 1000 milli-units (`quantity_milli`).
pub const M: i64 = 1000;
pub const DAYS_PER_YEAR: i64 = 360;
pub const START_YEAR: i64 = 412;
pub const GOODS: [&str; 6] = ["grain", "fish", "ale", "cloth", "timber", "tools"];
pub const MONTHS: [&str; 12] = [
    "Thawmonth", "Seedmonth", "Greenmonth", "Highsun", "Goldmonth", "Reapmonth", "Fallmonth", "Mistmonth",
    "Frostmonth", "Deepwinter", "Icemonth", "Stirring",
];

pub fn date_string(day: i64) -> String {
    let y = START_YEAR + day.div_euclid(DAYS_PER_YEAR);
    let doy = day.rem_euclid(DAYS_PER_YEAR);
    format!("{} {} {}", doy % 30 + 1, MONTHS[(doy / 30) as usize], y)
}
pub fn season(day: i64) -> &'static str {
    match day.rem_euclid(DAYS_PER_YEAR) / 90 {
        0 => "spring",
        1 => "summer",
        2 => "autumn",
        _ => "winter",
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    pub world_id: Id,
    pub name: String,
    pub seed: u64,
    /// Version of the deterministic simulation rules these events were produced under.
    pub engine: String,
    /// Days elapsed since the world epoch on this branch.
    pub day: i64,
    pub town: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Traits {
    pub ambition: i64,
    pub empathy: i64,
    pub courage: i64,
    pub honesty: i64,
    pub sociability: i64,
    pub temper: i64,
    pub piety: i64,
    pub curiosity: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Stats {
    /// STR, DEX, CON, INT, WIS, CHA
    pub abilities: [i64; 6],
    pub level: i64,
    pub class: String,
    pub hp: i64,
    pub max_hp: i64,
    pub ac: i64,
    pub attack_bonus: i64,
    pub dmg_dice: i64,
    pub dmg_sides: i64,
    pub dmg_bonus: i64,
    #[serde(default)]
    pub exhaustion: i64,
    #[serde(default)]
    pub potions: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Memory {
    pub day: i64,
    pub kind: String,
    pub text: String,
    /// 0..100 — how strongly the memory was encoded.
    pub salience: i64,
    pub about: Vec<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Person {
    pub id: Id,
    pub name: String,
    pub family: String,
    pub sex: String,
    pub birth_day: i64,
    /// alive | dead | emigrated
    pub status: String,
    pub death_day: Option<i64>,
    pub household: Id,
    pub home: Id,
    pub work: Id,
    pub occupation: String,
    pub employer: Option<Id>,
    /// Cognition budget tier 1..4 (T0 cohorts are not persons).
    pub tier: u8,
    pub traits: Traits,
    pub health: i64,
    pub hunger: i64,
    pub parents: Vec<Id>,
    pub spouse: Option<Id>,
    pub children: Vec<Id>,
    pub stats: Stats,
    pub memories: Vec<Memory>,
    pub goals: Vec<String>,
    pub flags: Vec<String>,
    pub jailed_until: Option<i64>,
    pub is_pc: bool,
    /// Name of the agent profile (.md) that governs this character's cognition.
    pub agent_profile: String,
    pub talks_with_party: i64,
    #[serde(default)]
    pub appearance: String,
    #[serde(default)]
    pub languages: Vec<String>,
}

impl Person {
    pub fn alive(&self) -> bool {
        self.status == "alive"
    }
    pub fn age(&self, day: i64) -> i64 {
        (day - self.birth_day).div_euclid(DAYS_PER_YEAR)
    }
    pub fn full_name(&self) -> String {
        format!("{} {}", self.name, self.family)
    }
    pub fn remember(&mut self, day: i64, kind: &str, text: String, salience: i64, about: Vec<Id>) {
        self.memories.push(Memory { day, kind: kind.into(), text, salience, about });
        if self.memories.len() > 28 {
            // Forget the weakest trace: salience decays with age (reconstructive memory, Decision 17).
            let mut worst = 0usize;
            let mut worst_score = i64::MAX;
            for (i, m) in self.memories.iter().enumerate() {
                let score = m.salience * 10 - (day - m.day).max(0);
                if score < worst_score {
                    worst_score = score;
                    worst = i;
                }
            }
            self.memories.remove(worst);
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Household {
    pub id: Id,
    pub name: String,
    pub head: Id,
    pub members: Vec<Id>,
    pub home: Id,
    pub dissolved: bool,
    pub last_gouge_grievance: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Location {
    pub id: Id,
    pub name: String,
    pub kind: String,
    pub x: i64,
    pub y: i64,
    pub desc: String,
    /// (neighbour id, walking minutes)
    pub links: Vec<(Id, i64)>,
    pub residential: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Org {
    pub id: Id,
    pub name: String,
    pub kind: String,
    pub charter: String,
    pub leader: Option<Id>,
    pub members: Vec<Id>,
    pub seat: Id,
    /// Tunable policy integers (tax_permille, ferry_fee_milli, hoarding, wage_milli, ...).
    pub policy: BTreeMap<String, i64>,
    pub goals: Vec<String>,
    pub unpaid_days: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub id: Id,
    pub name: String,
    pub kind: String,
    pub owner: Id,
    pub location: Id,
    pub active: bool,
    pub condition: i64,
    pub protected: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Grievance {
    pub day: i64,
    pub what: String,
    pub weight: i64,
}

/// Directed social edge `from -> to`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    pub trust: i64,
    pub fondness: i64,
    pub respect: i64,
    pub fear: i64,
    pub familiarity: i64,
    pub kinds: Vec<String>,
    pub grievances: Vec<Grievance>,
    /// neutral | friend | ally | rival | enemy | nemesis
    pub status: String,
    pub last_scheme_day: i64,
}

impl Relation {
    pub fn hostility(&self, day: i64) -> i64 {
        self.grievances.iter().map(|g| (g.weight - (day - g.day).max(0) / 20).max(g.weight / 3)).sum()
    }
    pub fn affinity(&self) -> i64 {
        (self.trust + self.fondness) / 2
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct OrgRelation {
    pub score: i64,
    /// allied | friendly | neutral | tense | hostile
    pub status: String,
    pub history: Vec<(i64, String)>,
}

/// A proposition someone may believe. `truth` is GM-only objective fact.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Claim {
    pub id: Id,
    pub day: i64,
    /// sighting | rumour | notice | slander | accusation | lore
    pub kind: String,
    pub subject: Option<Id>,
    pub crime: Option<Id>,
    pub text: String,
    pub truth: bool,
    pub origin: Option<Id>,
    pub parent: Option<Id>,
    pub public: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Know {
    pub confidence: i64,
    pub day: i64,
    pub source: Option<Id>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Crime {
    pub id: Id,
    pub day: i64,
    pub kind: String,
    /// Objective truth. Never included in player or NPC views.
    pub culprit: Id,
    pub victim: Id,
    pub location: Id,
    pub commodity: String,
    pub amount: i64,
    pub motive: String,
    /// open | solved | wrongful | cold
    pub status: String,
    pub accused: Option<Id>,
    pub suspicion: BTreeMap<Id, i64>,
    pub evidence: Vec<Id>,
    pub closed_day: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    pub id: Id,
    pub crime: Id,
    pub kind: String,
    pub location: Id,
    pub desc: String,
    pub points_to: Option<Id>,
    pub quality: i64,
    pub found_by_watch: bool,
    pub found_by_party: bool,
    pub custody: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Fact {
    pub id: Id,
    pub day: i64,
    pub text: String,
    pub protected: bool,
    pub gm_only: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Shock {
    pub kind: String,
    pub until_day: i64,
    pub severity: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Weather {
    pub rain: i64,
    pub temp: i64,
    pub river: i64,
    pub growth_sum: i64,
    pub growth_days: i64,
    pub last_growth_index: i64,
    pub shocks: Vec<Shock>,
    pub desc: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DayStat {
    pub day: i64,
    pub prices: BTreeMap<String, i64>,
    pub volume: BTreeMap<String, i64>,
    pub population: i64,
    pub hunger: i64,
    pub crimes: i64,
    pub coin_households: i64,
    pub grain_stock: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Market {
    pub prices: BTreeMap<String, i64>,
    pub base: BTreeMap<String, i64>,
    pub foreign: BTreeMap<String, i64>,
    pub history: Vec<DayStat>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Flow {
    pub issued: i64,
    pub produced: i64,
    pub consumed: i64,
    pub spoiled: i64,
}

/// Hard-magic ledger for the primary reality. Potential is conserved:
/// `ambient + bound + dissipated == capacity_total` unless an explicit
/// `magic.source_opened` event changes `capacity_total` (AT-019).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MagicField {
    pub law_version: String,
    pub ambient: i64,
    pub bound: i64,
    pub dissipated: i64,
    pub capacity_total: i64,
    /// Fraction (permille per day) of dissipated potential that relaxes back to ambient.
    pub relax_permille: i64,
    pub spells_cast: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BibleEntry {
    pub title: String,
    pub body: String,
    /// authored_fixture | generated | emergent
    pub provenance: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct JournalEntry {
    pub day: i64,
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Line {
    pub who: String,
    pub text: String,
    pub day: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Party {
    pub members: Vec<Id>,
    pub location: Id,
    pub minute: i64,
    pub knowledge: BTreeMap<Id, Know>,
    pub journal: Vec<JournalEntry>,
    pub conversations: BTreeMap<Id, Vec<Line>>,
    pub reputation: BTreeMap<Id, i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Combatant {
    pub id: Id,
    pub name: String,
    pub side: String,
    pub hp: i64,
    pub max_hp: i64,
    pub ac: i64,
    pub init: i64,
    pub attack_bonus: i64,
    pub dmg_dice: i64,
    pub dmg_sides: i64,
    pub dmg_bonus: i64,
    pub dodging: bool,
    pub fled: bool,
    pub down: bool,
    pub surprised: bool,
    pub is_pc: bool,
    pub courage: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Encounter {
    pub active: bool,
    pub round: i64,
    pub turn: usize,
    pub combatants: Vec<Combatant>,
    pub log: Vec<String>,
    pub nonlethal: bool,
    pub outcome: String,
    pub location: Id,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Campaign {
    pub name: String,
    /// srd-5.1 (2014) | srd-5.2.1 (2024)
    pub rules: String,
    /// human | ai | collaborative
    pub gm_mode: String,
    /// category -> excluded | veil | limited | allowed
    pub content: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct WorldState {
    pub meta: Meta,
    pub campaign: Option<Campaign>,
    pub bible: BTreeMap<String, Vec<BibleEntry>>,
    pub locations: BTreeMap<Id, Location>,
    pub persons: BTreeMap<Id, Person>,
    pub households: BTreeMap<Id, Household>,
    pub orgs: BTreeMap<Id, Org>,
    pub assets: BTreeMap<Id, Asset>,
    /// Ledger: "owner|commodity" -> milli-units. Never negative except `abroad`.
    pub accounts: BTreeMap<String, i64>,
    pub flows: BTreeMap<String, Flow>,
    pub market: Market,
    pub weather: Weather,
    /// "from|to" -> relation
    pub relations: BTreeMap<String, Relation>,
    /// "a|b" with a < b
    pub org_relations: BTreeMap<String, OrgRelation>,
    pub claims: BTreeMap<Id, Claim>,
    /// person -> claim -> knowledge
    pub knowledge: BTreeMap<Id, BTreeMap<Id, Know>>,
    pub crimes: BTreeMap<Id, Crime>,
    pub evidence: BTreeMap<Id, Evidence>,
    pub facts: BTreeMap<Id, Fact>,
    pub magic: MagicField,
    pub party: Party,
    pub encounter: Option<Encounter>,
    pub counters: BTreeMap<String, u64>,
    pub hinterland_pop: i64,
}

#[derive(Clone, Debug)]
pub struct Reject {
    pub code: String,
    pub message: String,
}
pub type Res<T> = Result<T, Reject>;
pub fn reject<T>(code: &str, message: impl Into<String>) -> Res<T> {
    Err(Reject { code: code.into(), message: message.into() })
}
impl std::fmt::Display for Reject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

pub fn rel_key(a: &str, b: &str) -> String {
    format!("{a}|{b}")
}
pub fn org_key(a: &str, b: &str) -> String {
    if a < b {
        format!("{a}|{b}")
    } else {
        format!("{b}|{a}")
    }
}

impl WorldState {
    pub fn next_id(&mut self, prefix: &str) -> Id {
        let c = self.counters.entry(prefix.to_string()).or_insert(0);
        *c += 1;
        format!("{prefix}{:04}", *c)
    }
    pub fn bump(&mut self, counter: &str) -> u64 {
        let c = self.counters.entry(counter.to_string()).or_insert(0);
        *c += 1;
        *c
    }
    pub fn acct(&self, owner: &str, commodity: &str) -> i64 {
        *self.accounts.get(&format!("{owner}|{commodity}")).unwrap_or(&0)
    }
    fn add(&mut self, owner: &str, commodity: &str, delta: i64) {
        *self.accounts.entry(format!("{owner}|{commodity}")).or_insert(0) += delta;
    }
    /// Atomic debit/credit. `abroad` is the only owner allowed to go negative
    /// (it stands for the rest of the world and keeps coin and goods conserved).
    pub fn transfer(&mut self, from: &str, to: &str, commodity: &str, amount: i64) -> Res<()> {
        if amount < 0 {
            return reject("invalid_amount", "transfer amount must be non-negative");
        }
        if amount == 0 || from == to {
            return Ok(());
        }
        if from != "abroad" && self.acct(from, commodity) < amount {
            return reject(
                "insufficient_balance",
                format!("{from} holds {} milli-{commodity}, needs {amount}", self.acct(from, commodity)),
            );
        }
        self.add(from, commodity, -amount);
        self.add(to, commodity, amount);
        Ok(())
    }
    /// Transfer as much as possible up to `amount`; returns what moved.
    pub fn transfer_up_to(&mut self, from: &str, to: &str, commodity: &str, amount: i64) -> i64 {
        let amt = if from == "abroad" { amount } else { amount.min(self.acct(from, commodity)) }.max(0);
        let _ = self.transfer(from, to, commodity, amt);
        amt
    }
    pub fn issue(&mut self, owner: &str, commodity: &str, amount: i64) {
        self.add(owner, commodity, amount);
        self.flows.entry(commodity.into()).or_default().issued += amount;
    }
    pub fn produce(&mut self, owner: &str, commodity: &str, amount: i64) {
        self.add(owner, commodity, amount);
        self.flows.entry(commodity.into()).or_default().produced += amount;
    }
    pub fn consume(&mut self, owner: &str, commodity: &str, amount: i64) -> i64 {
        let amt = amount.min(self.acct(owner, commodity)).max(0);
        self.add(owner, commodity, -amt);
        self.flows.entry(commodity.into()).or_default().consumed += amt;
        amt
    }
    pub fn spoil(&mut self, owner: &str, commodity: &str, amount: i64) {
        let amt = amount.min(self.acct(owner, commodity)).max(0);
        self.add(owner, commodity, -amt);
        self.flows.entry(commodity.into()).or_default().spoiled += amt;
    }
    /// Conservation residual per commodity: 0 means every unit is accounted for.
    pub fn conservation_residual(&self) -> BTreeMap<String, i64> {
        let mut sums: BTreeMap<String, i64> = BTreeMap::new();
        for (k, v) in &self.accounts {
            let c = k.rsplit('|').next().unwrap_or("").to_string();
            *sums.entry(c).or_insert(0) += v;
        }
        let mut out = BTreeMap::new();
        let mut keys: Vec<String> = sums.keys().cloned().collect();
        for k in self.flows.keys() {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
        for c in keys {
            let f = self.flows.get(&c).cloned().unwrap_or_default();
            let expected = f.issued + f.produced - f.consumed - f.spoiled;
            out.insert(c.clone(), sums.get(&c).copied().unwrap_or(0) - expected);
        }
        out
    }
    pub fn negative_accounts(&self) -> Vec<String> {
        self.accounts.iter().filter(|(k, v)| **v < 0 && !k.starts_with("abroad|")).map(|(k, _)| k.clone()).collect()
    }
    pub fn rel(&self, from: &str, to: &str) -> Option<&Relation> {
        self.relations.get(&rel_key(from, to))
    }
    pub fn rel_mut(&mut self, from: &str, to: &str) -> &mut Relation {
        self.relations.entry(rel_key(from, to)).or_insert_with(|| Relation {
            trust: 40,
            fondness: 40,
            respect: 40,
            status: "neutral".into(),
            last_scheme_day: -1000,
            ..Default::default()
        })
    }
    pub fn living(&self) -> impl Iterator<Item = &Person> {
        self.persons.values().filter(|p| p.alive())
    }
    pub fn population(&self) -> i64 {
        self.living().filter(|p| !p.is_pc).count() as i64
    }
    pub fn person_name(&self, id: &str) -> String {
        self.persons.get(id).map(|p| p.full_name()).or_else(|| self.orgs.get(id).map(|o| o.name.clone())).unwrap_or_else(|| id.to_string())
    }
    pub fn owner_name(&self, id: &str) -> String {
        if let Some(h) = self.households.get(id) {
            return h.name.clone();
        }
        self.person_name(id)
    }
    pub fn learn(&mut self, person: &str, claim: &str, confidence: i64, source: Option<Id>) -> bool {
        let day = self.meta.day;
        let k = self.knowledge.entry(person.to_string()).or_default();
        match k.get_mut(claim) {
            Some(existing) => {
                if confidence > existing.confidence {
                    existing.confidence = confidence;
                }
                false
            }
            None => {
                k.insert(claim.to_string(), Know { confidence, day, source });
                true
            }
        }
    }
    /// Does `person` know `claim`? Public notices become common knowledge two days after posting.
    pub fn knows(&self, person: &str, claim: &str) -> Option<i64> {
        if let Some(k) = self.knowledge.get(person).and_then(|k| k.get(claim)) {
            return Some(k.confidence);
        }
        match self.claims.get(claim) {
            Some(c) if c.public && self.meta.day >= c.day + 2 => Some(80),
            _ => None,
        }
    }
    /// Every claim id a person can draw on, with confidence.
    pub fn known_claims(&self, person: &str) -> Vec<(Id, i64)> {
        let mut v: Vec<(Id, i64)> = self.knowledge.get(person).map(|k| k.iter().map(|(c, kn)| (c.clone(), kn.confidence)).collect()).unwrap_or_default();
        for c in self.claims.values() {
            if c.public && self.meta.day >= c.day + 2 && !v.iter().any(|x| x.0 == c.id) {
                v.push((c.id.clone(), 80));
            }
        }
        v
    }
    pub fn add_claim(&mut self, mut c: Claim) -> Id {
        let id = self.next_id("clm");
        c.id = id.clone();
        c.day = self.meta.day;
        self.claims.insert(id.clone(), c);
        id
    }
    pub fn org_of(&self, person: &str) -> Option<&Org> {
        self.orgs.values().find(|o| o.members.iter().any(|m| m == person))
    }
    pub fn grain_price_ratio_pct(&self) -> i64 {
        let p = *self.market.prices.get("grain").unwrap_or(&1);
        let b = *self.market.base.get("grain").unwrap_or(&1);
        p * 100 / b.max(1)
    }
}
