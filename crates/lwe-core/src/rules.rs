//! D&D 5e-compatible resolution core, pinned per campaign to a rules profile
//! (Decision 12): `srd-5.1` (2014) or `srd-5.2.1` (2024). Only mechanics from
//! the System Reference Documents (CC BY 4.0, see docs/LICENSES.md) are used,
//! and only a small, tested subset is implemented. Every roll comes from a
//! seeded audit stream so combat replays exactly.

use crate::model::*;
use crate::rng::Rng;
use serde_json::{json, Value};

pub const PROFILES: [&str; 2] = ["srd-5.1", "srd-5.2.1"];

pub fn ability_mod(score: i64) -> i64 {
    (score - 10).div_euclid(2)
}
pub fn proficiency(level: i64) -> i64 {
    2 + (level.max(1) - 1) / 4
}
pub fn profile(s: &WorldState) -> String {
    s.campaign.as_ref().map(|c| c.rules.clone()).unwrap_or_else(|| "srd-5.2.1".into())
}

/// Skill -> ability index (STR 0, DEX 1, CON 2, INT 3, WIS 4, CHA 5).
pub fn skill_ability(skill: &str) -> Option<usize> {
    Some(match skill {
        "athletics" | "strength" => 0,
        "acrobatics" | "sleight_of_hand" | "stealth" | "dexterity" => 1,
        "constitution" => 2,
        "arcana" | "history" | "investigation" | "nature" | "religion" | "intelligence" => 3,
        "animal_handling" | "insight" | "medicine" | "perception" | "survival" | "wisdom" => 4,
        "deception" | "intimidation" | "performance" | "persuasion" | "charisma" => 5,
        _ => return None,
    })
}

#[derive(Clone, Debug)]
pub struct D20 {
    pub rolls: Vec<i64>,
    pub natural: i64,
    pub modifier: i64,
    pub total: i64,
    pub mode: &'static str,
}
impl D20 {
    pub fn json(&self) -> Value {
        json!({"rolls": self.rolls, "natural": self.natural, "modifier": self.modifier, "total": self.total, "mode": self.mode})
    }
}

fn dice_stream(s: &mut WorldState) -> Rng {
    let n = s.bump("dice");
    Rng::new(s.meta.seed, "dice", n, 0)
}

/// `adv`: >0 advantage, <0 disadvantage, 0 straight. Advantage and disadvantage cancel.
pub fn d20(s: &mut WorldState, modifier: i64, adv: i64) -> D20 {
    let mut r = dice_stream(s);
    let a = r.die(20);
    let (rolls, natural, mode) = if adv == 0 {
        (vec![a], a, "straight")
    } else {
        let b = r.die(20);
        if adv > 0 {
            (vec![a, b], a.max(b), "advantage")
        } else {
            (vec![a, b], a.min(b), "disadvantage")
        }
    };
    D20 { rolls, natural, modifier, total: natural + modifier, mode }
}

pub fn roll_dice(s: &mut WorldState, n: i64, sides: i64) -> (i64, Vec<i64>) {
    let mut r = dice_stream(s);
    let rolls: Vec<i64> = (0..n.max(0)).map(|_| r.die(sides)).collect();
    (rolls.iter().sum(), rolls)
}

/// Exhaustion differs by edition. Returns (flat modifier, adv shift) for a d20 test.
/// 2024 (SRD 5.2.1): every d20 test takes −2 per exhaustion level.
/// 2014 (SRD 5.1): level 1+ disadvantage on ability checks; level 3+ on attacks and saves.
pub fn exhaustion_effect(profile: &str, level: i64, kind: &str) -> (i64, i64) {
    if level <= 0 {
        return (0, 0);
    }
    if profile == "srd-5.2.1" {
        (-2 * level, 0)
    } else {
        match kind {
            "check" => (0, -1),
            _ if level >= 3 => (0, -1),
            _ => (0, 0),
        }
    }
}

/// Drinking a potion of healing: an action in 2014, a Bonus Action in 2024.
pub fn potion_uses_action(profile: &str) -> bool {
    profile != "srd-5.2.1"
}

pub fn skill_check(s: &mut WorldState, pc: &str, skill: &str, dc: i64, adv: i64) -> Res<(D20, bool)> {
    let Some(ai) = skill_ability(skill) else { return reject("unknown_skill", format!("unknown skill '{skill}'")) };
    let Some(p) = s.persons.get(pc) else { return reject("unknown_entity", "no such character") };
    let prof = if class_skills(&p.stats.class).contains(&skill) { proficiency(p.stats.level) } else { 0 };
    let (flat, shift) = exhaustion_effect(&profile(s), p.stats.exhaustion, "check");
    let modifier = ability_mod(p.stats.abilities[ai]) + prof + flat;
    let roll = d20(s, modifier, adv + shift);
    let ok = roll.total >= dc;
    Ok((roll, ok))
}

pub fn class_skills(class: &str) -> &'static [&'static str] {
    match class {
        "fighter" => &["athletics", "intimidation", "perception"],
        "rogue" => &["stealth", "sleight_of_hand", "investigation", "deception", "perception", "insight"],
        "wizard" => &["arcana", "history", "investigation"],
        "cleric" => &["insight", "medicine", "persuasion", "religion"],
        "guard" => &["perception", "insight", "investigation"],
        _ => &[],
    }
}

pub fn make_pc(class: &str, abilities: Option<[i64; 6]>) -> Res<Stats> {
    // standard array assigned by class priority unless explicit scores are given
    let order: [usize; 6] = match class {
        "fighter" => [0, 2, 1, 4, 5, 3],
        "rogue" => [1, 3, 2, 5, 4, 0],
        "wizard" => [3, 1, 2, 4, 5, 0],
        "cleric" => [4, 0, 2, 5, 1, 3],
        _ => return reject("unknown_class", "class must be fighter, rogue, wizard or cleric"),
    };
    let ab = match abilities {
        Some(a) => {
            if a.iter().any(|x| !(3..=18).contains(x)) {
                return reject("invalid_ability", "ability scores must be 3..18");
            }
            a
        }
        None => {
            let arr = [15, 14, 13, 12, 10, 8];
            let mut a = [0; 6];
            for (i, idx) in order.iter().enumerate() {
                a[*idx] = arr[i];
            }
            a
        }
    };
    let (hd, ac, atk_ab, dd, ds) = match class {
        "fighter" => (10, 16, 0, 1, 8),
        "rogue" => (8, 11 + ability_mod(ab[1]), 1, 1, 6),
        "wizard" => (6, 10 + ability_mod(ab[1]), 3, 1, 10),
        _ => (8, 16, 0, 1, 6),
    };
    let hp = hd + ability_mod(ab[2]);
    Ok(Stats { abilities: ab, level: 1, class: class.into(), hp, max_hp: hp, ac, attack_bonus: proficiency(1) + ability_mod(ab[atk_ab]), dmg_dice: dd, dmg_sides: ds, dmg_bonus: if class == "wizard" { 0 } else { ability_mod(ab[atk_ab]) }, exhaustion: 0, potions: 1 })
}

#[derive(Clone, Debug)]
pub struct AttackResult {
    pub hit: bool,
    pub crit: bool,
    pub damage: i64,
    pub roll: D20,
}

/// Attack roll vs AC. Natural 20 always hits and doubles the damage dice; natural 1 always misses.
pub fn attack(s: &mut WorldState, bonus: i64, ac: i64, dice: i64, sides: i64, dmg_bonus: i64, adv: i64) -> AttackResult {
    let roll = d20(s, bonus, adv);
    let crit = roll.natural == 20;
    let hit = crit || (roll.natural != 1 && roll.total >= ac);
    let damage = if hit { (roll_dice(s, if crit { dice * 2 } else { dice }, sides).0 + dmg_bonus).max(1) } else { 0 };
    AttackResult { hit, crit, damage, roll }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modifiers_and_proficiency() {
        assert_eq!(ability_mod(8), -1);
        assert_eq!(ability_mod(10), 0);
        assert_eq!(ability_mod(15), 2);
        assert_eq!(ability_mod(20), 5);
        assert_eq!(proficiency(1), 2);
        assert_eq!(proficiency(5), 3);
        assert_eq!(proficiency(17), 6);
    }
    #[test]
    fn edition_differences_are_pinned() {
        // AT-014: the two rules profiles give different, correct outcomes for the same fixture.
        assert_eq!(exhaustion_effect("srd-5.2.1", 2, "attack"), (-4, 0));
        assert_eq!(exhaustion_effect("srd-5.1", 2, "attack"), (0, 0));
        assert_eq!(exhaustion_effect("srd-5.1", 1, "check"), (0, -1));
        assert_eq!(exhaustion_effect("srd-5.1", 3, "save"), (0, -1));
        assert!(potion_uses_action("srd-5.1"));
        assert!(!potion_uses_action("srd-5.2.1"));
    }
    #[test]
    fn dice_are_auditable_and_replayable() {
        let mut a = WorldState::default();
        a.meta.seed = 9;
        let mut b = a.clone();
        let x: Vec<i64> = (0..20).map(|_| d20(&mut a, 3, 1).total).collect();
        let y: Vec<i64> = (0..20).map(|_| d20(&mut b, 3, 1).total).collect();
        assert_eq!(x, y);
        assert_eq!(a.counters["dice"], 20);
    }
    #[test]
    fn pc_creation() {
        let f = make_pc("fighter", None).unwrap();
        assert_eq!(f.abilities[0], 15);
        assert_eq!(f.max_hp, 10 + 2);
        assert_eq!(f.attack_bonus, 4);
        assert!(make_pc("bard", None).is_err());
        assert!(make_pc("rogue", Some([19, 10, 10, 10, 10, 10])).is_err());
    }
}
