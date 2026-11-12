//! The deterministic daily step (Decision 56). `step_day` is a pure function of
//! the prior state: all randomness comes from seeded streams keyed by day and
//! entity, all quantities are integers, and every flow goes through the ledger.
//!
//! Model status: these are simplified, explicitly-assumed synthetic models
//! (spec §12). They are internally consistent and conserved; they are not
//! calibrated against any real economy.

pub mod social;

use crate::model::*;
use crate::rng::{hash_str, Rng};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Incident {
    pub kind: String,
    pub text: String,
    pub data: Value,
    /// false => GM-private (contains objective truth players must not see).
    pub public: bool,
}

pub fn inc(out: &mut Vec<Incident>, kind: &str, public: bool, text: String, data: Value) {
    out.push(Incident { kind: kind.into(), text, data, public });
}

pub fn food_need(p: &Person, day: i64) -> i64 {
    if !p.alive() {
        0
    } else if p.age(day) < 14 {
        600
    } else {
        1000
    }
}

fn clamp(v: i64, lo: i64, hi: i64) -> i64 {
    v.max(lo).min(hi)
}

pub fn can_work(p: &Person, day: i64) -> bool {
    p.alive() && !p.is_pc && p.jailed_until.is_none() && p.health > 15 && (15..65).contains(&p.age(day))
}

/// Fraction (permille) of normal river traffic that can move today.
pub fn navigability(s: &WorldState) -> i64 {
    let w = &s.weather;
    if w.temp < -4 || w.river < 12 || w.river > 92 {
        return 0;
    }
    if s.orgs.get("org_lodge").map(|o| o.policy.get("strike").copied().unwrap_or(0)).unwrap_or(0) > 0 {
        return 0;
    }
    let ferry = s.assets.get("asset_ferry").map(|a| a.active).unwrap_or(false);
    let mut n = if w.river < 25 {
        400
    } else if w.river > 80 {
        500
    } else {
        1000
    };
    if !ferry {
        n /= 2;
    }
    n
}

pub fn step_day(s: &mut WorldState) -> Vec<Incident> {
    s.meta.day += 1;
    let mut out = vec![];
    weather(s, &mut out);
    production(s);
    let volumes = market(s);
    consumption(s);
    finance(s, &mut out);
    let crimes_before = s.crimes.len();
    social::crime(s, &mut out);
    social::investigate(s, &mut out);
    social::interactions(s, &mut out);
    social::rumours(s, &mut out);
    if s.meta.day % 7 == 0 {
        social::schemes(s, &mut out);
        social::org_politics(s, &mut out);
        social::courtship(s, &mut out);
        social::refresh_tiers(s);
    }
    social::demography(s, &mut out);
    // magic: dissipated potential relaxes back to ambient (conserved)
    let relax = s.magic.dissipated * s.magic.relax_permille / 1000;
    s.magic.dissipated -= relax;
    s.magic.ambient += relax;
    record_stats(s, volumes, (s.crimes.len() - crimes_before) as i64);
    out
}

fn weather(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let doy = day.rem_euclid(DAYS_PER_YEAR);
    // Weather is keyed by day only, so sibling branches share the same sky (counterfactuals, Decision 57).
    let mut r = Rng::new(s.meta.seed, "weather", day as u64, 0);
    let (base_rain, base_temp) = match doy / 90 {
        0 => (50, 10),
        1 => (38, 21),
        2 => (50, 11),
        _ => (42, 0),
    };
    let mut rain = clamp(r.bell(base_rain, 45), 0, 120);
    let mut temp = base_temp + r.bell(0, 9);
    s.weather.shocks.retain(|k| k.until_day >= day);
    for k in s.weather.shocks.clone() {
        match k.kind.as_str() {
            "drought" => rain = rain * (100 - k.severity).max(0) / 100,
            "flood" => rain = clamp(rain + k.severity, 0, 140),
            "frost" => temp -= 18,
            _ => {}
        }
    }
    let melt = if doy < 40 { 14 } else { 0 };
    let target = clamp(rain * 11 / 10 + melt, 0, 110);
    let river = clamp((s.weather.river * 88 + target * 12) / 100, 0, 100);
    if (30..150).contains(&doy) {
        let dist = if rain < 30 { 30 - rain } else if rain > 70 { rain - 70 } else { 0 };
        let mut score = clamp(100 - dist * 3, 5, 100);
        if river > 92 {
            score /= 2;
        }
        s.weather.growth_sum += score;
        s.weather.growth_days += 1;
    }
    if doy == 150 {
        let idx = if s.weather.growth_days > 0 { s.weather.growth_sum / s.weather.growth_days } else { 100 };
        s.weather.last_growth_index = idx;
        s.weather.growth_sum = 0;
        s.weather.growth_days = 0;
        let verdict = if idx < 55 { "The harvest has failed" } else if idx < 75 { "A thin harvest begins" } else { "A good harvest begins" };
        inc(out, if idx < 55 { "harvest.failed" } else { "harvest.began" }, true, format!("{verdict} in the Sable valley (growth index {idx}/100)."), json!({"growth_index": idx}));
    }
    let was_nav = navigability(s) > 0;
    s.weather.rain = rain;
    s.weather.temp = temp;
    s.weather.river = river;
    s.weather.desc = format!(
        "{}{}{}",
        if temp < -4 { "hard frost, " } else if temp < 5 { "cold, " } else if temp > 24 { "hot, " } else { "mild, " },
        if rain < 15 { "dry" } else if rain < 55 { "showery" } else if rain < 90 { "heavy rain" } else { "torrential rain" },
        if river < 12 { "; the Sable is too low for barges" } else if river > 92 { "; the Sable is in flood" } else { "" }
    );
    let now_nav = navigability(s) > 0;
    if was_nav != now_nav {
        let text = if now_nav { "The river is open to barges again.".to_string() } else { format!("River traffic has stopped ({}).", s.weather.desc) };
        inc(out, "river.traffic", true, text, json!({"navigable": now_nav, "river": river, "temp": temp}));
    }
}

fn production(s: &mut WorldState) {
    let day = s.meta.day;
    let doy = day.rem_euclid(DAYS_PER_YEAR);
    let winter = doy >= 270;
    let river_ok = (25..=80).contains(&s.weather.river);
    let idx = s.weather.last_growth_index;
    let workers: Vec<(Id, Id, String)> = s.persons.values().filter(|p| can_work(p, day)).map(|p| (p.id.clone(), p.household.clone(), p.occupation.clone())).collect();
    for (pid, hh, occ) in workers {
        let mut r = Rng::new(s.meta.seed, "prod", day as u64, hash_str(&pid));
        let var = r.range(80, 120);
        match occ.as_str() {
            "farmer" if (150..180).contains(&doy) => s.produce(&hh, "grain", 70 * M * idx / 100 * var / 100),
            "fisher" => {
                let mut q = 3 * M * var / 100;
                if !river_ok {
                    q = q * 4 / 10;
                }
                if winter {
                    q = q * 7 / 10;
                }
                s.produce(&hh, "fish", q)
            }
            "woodcutter" => s.produce(&hh, "timber", if winter { 2 } else { 3 } * M * var / 100),
            "weaver" => s.produce(&hh, "cloth", 250 * var / 100),
            "smith" => {
                if s.consume(&hh, "timber", M) == M {
                    s.produce(&hh, "tools", 200 * var / 100)
                }
            }
            "brewer" => {
                if s.consume(&hh, "grain", 5 * M) == 5 * M {
                    s.produce(&hh, "ale", 6 * M * var / 100)
                }
            }
            _ => {}
        }
    }
    if (150..180).contains(&doy) {
        // Tier-0 cohort: ~900 villagers of the hinterland, simulated as one account.
        s.produce("hinterland", "grain", 24000 * M / 30 * idx / 100 * s.hinterland_pop / 900);
    }
}

fn household_need(s: &WorldState, h: &Household) -> i64 {
    h.members.iter().filter_map(|m| s.persons.get(m)).map(|p| food_need(p, s.meta.day)).sum()
}

/// Clear one commodity at the posted price with proportional rationing.
/// Returns (volume, demand, supply). Goods and coin are exactly conserved.
fn clear(s: &mut WorldState, good: &str, sellers: &[(Id, i64)], buyers: &[(Id, i64)], tax_permille: i64) -> (i64, i64, i64) {
    let price = *s.market.prices.get(good).unwrap_or(&1000);
    let sellers: Vec<(Id, i64)> = sellers.iter().map(|(o, q)| (o.clone(), (*q).min(s.acct(o, good)).max(0))).filter(|x| x.1 > 0).collect();
    let buyers: Vec<(Id, i64)> = buyers.iter().map(|(o, q)| (o.clone(), (*q).min(s.acct(o, "coin") * 1000 / price.max(1)).max(0))).filter(|x| x.1 > 0).collect();
    let supply: i64 = sellers.iter().map(|x| x.1).sum();
    let demand: i64 = buyers.iter().map(|x| x.1).sum();
    if supply == 0 || demand == 0 {
        return (0, demand, supply);
    }
    let vol = supply.min(demand);
    let mut allocs: Vec<(Id, i64)> = vec![];
    let mut bought = 0;
    let mut coin_pool = 0;
    for (b, w) in &buyers {
        let q = (*w as i128 * vol as i128 / demand as i128) as i64;
        if q <= 0 {
            continue;
        }
        let cost = q * price / 1000;
        if s.transfer(b, "market", "coin", cost).is_ok() {
            coin_pool += cost;
            bought += q;
            allocs.push((b.clone(), q));
        }
    }
    if bought == 0 {
        return (0, demand, supply);
    }
    // sellers deliver pro rata; remainder taken in order from sellers with stock left
    let mut sold: Vec<(Id, i64)> = sellers.iter().map(|(o, q)| (o.clone(), (*q as i128 * bought as i128 / supply as i128) as i64)).collect();
    let mut rem = bought - sold.iter().map(|x| x.1).sum::<i64>();
    for (i, (_, q)) in sellers.iter().enumerate() {
        if rem == 0 {
            break;
        }
        let extra = (q - sold[i].1).min(rem);
        sold[i].1 += extra;
        rem -= extra;
    }
    for (o, q) in &sold {
        let _ = s.transfer(o, "market", good, *q);
    }
    for (b, q) in &allocs {
        let _ = s.transfer("market", b, good, *q);
    }
    let tax = coin_pool * tax_permille / 1000;
    let _ = s.transfer("market", "org_reeve", "coin", tax);
    let net = coin_pool - tax;
    let mut paid = 0;
    let last = sold.iter().rposition(|x| x.1 > 0);
    for (i, (o, q)) in sold.iter().enumerate() {
        let share = if Some(i) == last { net - paid } else { (net as i128 * *q as i128 / bought as i128) as i64 };
        let _ = s.transfer("market", o, "coin", share);
        paid += share;
    }
    (bought, demand, supply)
}

fn market(s: &mut WorldState) -> BTreeMap<String, i64> {
    let day = s.meta.day;
    let doy = day.rem_euclid(DAYS_PER_YEAR);
    let nav = navigability(s);
    let tax = s.orgs["org_reeve"].policy.get("tax_permille").copied().unwrap_or(80);
    let fee = s.orgs["org_lodge"].policy.get("ferry_fee_milli").copied().unwrap_or(150);
    let hoarding = s.orgs["org_compact"].policy.get("hoarding").copied().unwrap_or(0) > 0;
    let edict = s.orgs["org_reeve"].policy.get("edict_antihoarding").copied().unwrap_or(0) > 0;
    let barges = s.assets.values().filter(|a| a.kind == "barge" && a.active).count() as i64;
    let embargo = s.org_relations.get(&org_key("org_lodge", "org_reeve")).map(|r| r.status == "allied").unwrap_or(false) && edict;
    let mut capacity = s.orgs["org_compact"].policy.get("barge_capacity").copied().unwrap_or(160 * M) * nav / 1000 * barges / 2;
    if embargo {
        capacity /= 2;
    }
    if day % 7 == 0 {
        let mut r = Rng::new(s.meta.seed, "foreign", day as u64, 0);
        for g in GOODS {
            let base = s.market.base[g];
            let f = s.market.foreign[g];
            s.market.foreign.insert(g.into(), clamp(f * (1000 + r.range(-45, 45)) / 1000, base * 6 / 10, base * 17 / 10));
        }
    }
    let days_to_harvest_end = if doy < 180 { 180 - doy } else { 540 - doy };
    let hhs: Vec<Household> = s.households.values().filter(|h| !h.dissolved && !h.members.is_empty()).cloned().collect();
    let gp = s.market.prices["grain"];
    let fp = s.market.prices["fish"];
    let fish_share = clamp(300 + (gp * 125 / 100 - fp) * 300 / gp.max(1), 100, 600);
    let mut volumes = BTreeMap::new();
    let mut light_capacity = if capacity > 0 { 20 * M } else { 0 };

    for good in GOODS {
        let price = s.market.prices[good];
        let foreign = s.market.foreign[good];
        let base = s.market.base[good];
        let light = good == "cloth" || good == "tools";
        // Producers hold stock back when the price is poor (reservation), and buyers of
        // comforts buy less when they are dear (elastic demand). Food demand is inelastic.
        let reserve_pct = if good == "fish" { 100 } else { clamp((price * 100 / base - 40) * 100 / 60, 0, 100) };
        let elastic_pct = clamp(base * 100 / price.max(1), 15, 160);
        let mut sellers: Vec<(Id, i64)> = vec![];
        let mut buyers: Vec<(Id, i64)> = vec![];
        for h in &hhs {
            let need = household_need(s, h);
            let occs: Vec<&str> = h.members.iter().filter_map(|m| s.persons.get(m)).filter(|p| can_work(p, day)).map(|p| p.occupation.as_str()).collect();
            let has = |o: &str| occs.iter().any(|x| *x == o);
            let n_workers = |o: &str| occs.iter().filter(|x| **x == o).count() as i64;
            let stock = s.acct(&h.id, good);
            let food_stock = s.acct(&h.id, "grain") + s.acct(&h.id, "fish");
            let coin = s.acct(&h.id, "coin");
            let comfortable = coin > need * 12 * gp / 1000;
            let people = h.members.len() as i64;
            match good {
                "grain" => {
                    if has("farmer") {
                        let reserve = need * days_to_harvest_end * 105 / 100;
                        if stock > reserve {
                            sellers.push((h.id.clone(), (stock - reserve) * 4 / 100 * reserve_pct / 100));
                        }
                    } else {
                        let want = (need * 5 - food_stock).max(0) * (1000 - fish_share) / 1000;
                        let brew = if has("brewer") { (15 * M * n_workers("brewer") - stock).max(0) } else { 0 };
                        if want + brew > 0 {
                            buyers.push((h.id.clone(), want + brew));
                        }
                    }
                }
                "fish" => {
                    if has("fisher") {
                        if stock > need * 2 {
                            sellers.push((h.id.clone(), stock - need * 2));
                        }
                    } else {
                        let want = (need * 5 - food_stock).max(0) * fish_share / 1000;
                        if want > 0 {
                            buyers.push((h.id.clone(), want));
                        }
                    }
                }
                "timber" => {
                    if has("woodcutter") {
                        if stock > 20 * M {
                            sellers.push((h.id.clone(), (stock - 20 * M) * reserve_pct / 100));
                        }
                    } else {
                        let target = if has("smith") { 8 * M * n_workers("smith") } else { 8 * M };
                        if stock < target / 2 {
                            buyers.push((h.id.clone(), target - stock));
                        }
                    }
                }
                "ale" => {
                    if has("brewer") || has("innkeeper") {
                        if stock > 5 * M {
                            sellers.push((h.id.clone(), (stock - 5 * M) * reserve_pct / 100));
                        }
                    } else if comfortable {
                        buyers.push((h.id.clone(), occs.len() as i64 * 220 * elastic_pct / 100));
                    }
                }
                "cloth" => {
                    if has("weaver") {
                        sellers.push((h.id.clone(), stock * reserve_pct / 100));
                    } else if comfortable {
                        buyers.push((h.id.clone(), people * 14 * elastic_pct / 100));
                    }
                }
                "tools" => {
                    if has("smith") {
                        sellers.push((h.id.clone(), stock * reserve_pct / 100));
                    } else if comfortable {
                        let users = n_workers("farmer") + n_workers("woodcutter") + n_workers("fisher") + n_workers("weaver");
                        if users > 0 {
                            buyers.push((h.id.clone(), users * 7 * elastic_pct / 100));
                        }
                    }
                }
                _ => {}
            }
        }
        // tier-0 hinterland cohort
        let hc = s.acct("hinterland", "coin");
        match good {
            "grain" => sellers.push(("hinterland".into(), s.acct("hinterland", "grain") * 7 / 1000 * reserve_pct / 100)),
            "ale" if hc > 300 * M => buyers.push(("hinterland".into(), 14 * M * elastic_pct / 100)),
            "cloth" if hc > 300 * M => buyers.push(("hinterland".into(), 1300 * elastic_pct / 100)),
            "tools" if hc > 300 * M => buyers.push(("hinterland".into(), 550 * elastic_pct / 100)),
            "timber" if hc > 300 * M => buyers.push(("hinterland".into(), 4 * M * elastic_pct / 100)),
            _ => {}
        }
        // the Compact: stock sales, hoarding, arbitrage against downstream prices
        let cstock = s.acct("org_compact", good);
        if good == "grain" {
            if edict {
                sellers.push(("org_compact".into(), cstock * 15 / 100));
            } else if hoarding && price >= base * 5 / 2 {
                sellers.push(("org_compact".into(), cstock * 8 / 100)); // profit-taking at the peak
            } else if hoarding {
                buyers.push(("org_compact".into(), 90 * M));
            } else if price >= foreign * 95 / 100 {
                sellers.push(("org_compact".into(), cstock * 5 / 100));
            }
        } else if cstock > 0 {
            sellers.push(("org_compact".into(), cstock));
        }
        let est_supply: i64 = sellers.iter().map(|x| x.1.min(s.acct(&x.0, good)).max(0)).sum();
        let est_demand: i64 = buyers.iter().map(|x| x.1).sum();
        let landed = foreign * 115 / 100 + fee;
        let cap = if light { light_capacity } else { capacity };
        if cap > 0 && price > landed && est_demand > est_supply && !(good == "grain" && hoarding && !edict && price < base * 5 / 2) {
            // import: buy downstream, pay freight to the Lodge, sell here
            let q = (est_demand - est_supply).min(cap).min(s.acct("org_compact", "coin") * 1000 / (foreign + fee).max(1) / 2);
            if q > 0 {
                let _ = s.transfer("org_compact", "abroad", "coin", q * foreign / 1000);
                let _ = s.transfer("abroad", "org_compact", good, q);
                s.transfer_up_to("org_compact", "org_lodge", "coin", q * fee / 1000);
                if light { light_capacity -= q } else { capacity -= q }
                sellers.push(("org_compact".into(), q));
            }
        }
        let floor_price = foreign * 85 / 100 - fee;
        let mut exporting = false;
        let cap = if light { light_capacity } else { capacity };
        if cap > 0 && price < floor_price && est_supply > est_demand {
            buyers.push(("org_compact".into(), (est_supply - est_demand).min(cap)));
            exporting = true;
        }
        let before = s.acct("org_compact", good);
        let (vol, demand, supply) = clear(s, good, &sellers, &buyers, tax);
        if exporting {
            let got = (s.acct("org_compact", good) - before).max(0).min(cap);
            if got > 0 {
                let _ = s.transfer("org_compact", "abroad", good, got);
                let _ = s.transfer("abroad", "org_compact", "coin", got * foreign / 1000);
                s.transfer_up_to("org_compact", "org_lodge", "coin", got * fee / 1000);
                if light { light_capacity -= got } else { capacity -= got }
            }
        }
        // the Compact trims an over-full granary downstream
        if good == "grain" && !hoarding && capacity > 0 && s.acct("org_compact", "grain") > 7000 * M {
            let q = (s.acct("org_compact", "grain") - 7000 * M).min(capacity);
            let _ = s.transfer("org_compact", "abroad", "grain", q);
            let _ = s.transfer("abroad", "org_compact", "coin", q * foreign * 9 / 10 / 1000);
            capacity -= q;
        }
        // price discovery: bounded daily adjustment on excess demand (spec §12 candidate rule, integer form)
        let adj = if supply == 0 && demand > 0 {
            50
        } else if demand == 0 {
            -25
        } else {
            clamp((demand - supply) as i128 as i64 * 70 / supply.max(1), -35, 50)
        };
        s.market.prices.insert(good.into(), clamp(price * (1000 + adj) / 1000, base / 5, base * 12));
        volumes.insert(good.to_string(), vol);
    }
    volumes
}

fn consumption(s: &mut WorldState) {
    let day = s.meta.day;
    let winter = day.rem_euclid(DAYS_PER_YEAR) >= 270;
    let hhs: Vec<Household> = s.households.values().filter(|h| !h.dissolved && !h.members.is_empty()).cloned().collect();
    for h in &hhs {
        let need = household_need(s, h);
        if need == 0 {
            continue;
        }
        // kin share food before anyone goes hungry (gift economy; strengthens bonds)
        if s.acct(&h.id, "grain") + s.acct(&h.id, "fish") < need {
            let kin: Vec<Id> = s.relations.iter().filter(|(k, r)| k.starts_with(&format!("{}|", h.head)) && r.kinds.iter().any(|x| x == "kin")).map(|(k, _)| k.split('|').nth(1).unwrap().to_string()).collect();
            for k in kin {
                let Some(kp) = s.persons.get(&k) else { continue };
                if !kp.alive() || kp.household == h.id {
                    continue;
                }
                let kh = kp.household.clone();
                let Some(khh) = s.households.get(&kh) else { continue };
                let kneed = household_need(s, khh);
                let spare = s.acct(&kh, "grain") - kneed * 10;
                if spare > 0 {
                    let gift = spare.min(need * 3) / 2;
                    let _ = s.transfer(&kh, &h.id, "grain", gift);
                    let head = h.head.clone();
                    let r = s.rel_mut(&head, &k);
                    r.fondness = clamp(r.fondness + 2, 0, 100);
                    r.trust = clamp(r.trust + 2, 0, 100);
                    break;
                }
            }
        }
        let mut eaten = s.consume(&h.id, "fish", need);
        eaten += s.consume(&h.id, "grain", need - eaten);
        let short_pct = (need - eaten) * 100 / need;
        for m in &h.members {
            let Some(p) = s.persons.get_mut(m) else { continue };
            if !p.alive() {
                continue;
            }
            if short_pct == 0 {
                p.hunger = clamp(p.hunger - 8, 0, 100);
            } else {
                p.hunger = clamp(p.hunger + short_pct / 9 + 1, 0, 100);
            }
            let frail = p.age(day) < 6 || p.age(day) > 64;
            if p.hunger > 60 {
                p.health -= if frail { 2 } else { 1 };
            } else if p.hunger < 20 && p.health < 90 && day % 3 == 0 {
                p.health += 1;
            }
        }
        s.consume(&h.id, "timber", if winter { 500 } else { 200 });
        let fish = s.acct(&h.id, "fish");
        s.spoil(&h.id, "fish", fish * 30 / 100);
        let ale = s.acct(&h.id, "ale");
        s.spoil(&h.id, "ale", ale / 100);
    }
    for o in ["org_compact", "org_reeve", "hinterland"] {
        let g = s.acct(o, "grain");
        s.spoil(o, "grain", g * 4 / 10000);
    }
}

fn finance(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let nav = navigability(s);
    let winter = day.rem_euclid(DAYS_PER_YEAR) >= 270;
    let mut r = Rng::new(s.meta.seed, "transit", day as u64, 0);
    let season = if winter { 60 } else { 100 };
    let barges = s.assets.values().filter(|a| a.kind == "barge" && a.active).count() as i64;
    // transit trade: the reason the town exists. Coin enters from the wider world.
    let _ = s.transfer("abroad", "org_compact", "coin", 175 * M * nav / 1000 * season / 100 * barges / 2 * r.range(80, 120) / 100);
    let _ = s.transfer("abroad", "org_lodge", "coin", 125 * M * nav / 1000 * season / 100 * r.range(80, 120) / 100);
    let _ = s.transfer("abroad", "org_reeve", "coin", 28 * M * nav / 1000 * season / 100); // bridge and quay tolls

    // wages
    let org_ids: Vec<Id> = s.orgs.keys().cloned().collect();
    for oid in &org_ids {
        let org = s.orgs[oid].clone();
        let base = org.policy.get("wage_milli").copied().unwrap_or(8000);
        let mut unpaid = false;
        for m in &org.members {
            let Some(p) = s.persons.get(m) else { continue };
            if !can_work(p, day) {
                continue;
            }
            let mut wage = if p.occupation == "factor" { org.policy.get("factor_wage_milli").copied().unwrap_or(12000) } else { base };
            if org.leader.as_ref() == Some(m) {
                wage += 5000;
            }
            let hh = p.household.clone();
            if s.transfer(oid, &hh, "coin", wage).is_err() {
                unpaid = true;
            }
        }
        let o = s.orgs.get_mut(oid).unwrap();
        o.unpaid_days = if unpaid { o.unpaid_days + 1 } else { 0 };
    }
    // day-labour and service trades
    let people: Vec<(Id, Id, String)> = s.persons.values().filter(|p| can_work(p, day)).map(|p| (p.id.clone(), p.household.clone(), p.occupation.clone())).collect();
    let payers: Vec<Id> = s.households.values().filter(|h| !h.dissolved && s.acct(&h.id, "coin") > 40 * M).map(|h| h.id.clone()).collect();
    for (pid, hh, occ) in &people {
        match occ.as_str() {
            "labourer" => {
                let payer = if hash_str(pid) % 2 == 0 { "org_lodge" } else { "org_compact" };
                if s.acct(payer, "coin") > if payer == "org_compact" { 4000 * M } else { 2500 * M } {
                    let _ = s.transfer(payer, hh, "coin", 6200);
                }
            }
            "innkeeper" | "healer" | "priest" | "mage" => {
                if !payers.is_empty() {
                    let each = 8500 / payers.len() as i64;
                    for p in &payers {
                        if p != hh {
                            s.transfer_up_to(p, hh, "coin", each);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    // hearth dues
    if day % 30 == 0 {
        let due = s.orgs["org_reeve"].policy.get("hearth_tax_milli").copied().unwrap_or(10000);
        let ids: Vec<Id> = s.households.values().filter(|h| !h.dissolved && !h.members.is_empty()).map(|h| h.id.clone()).collect();
        for h in ids {
            s.transfer_up_to(&h, "org_reeve", "coin", due);
        }
    }
    // imported wares: salt, spice, pottery, iron — coin leaves town in proportion to comfort
    if nav > 0 {
        let gp = s.market.prices["grain"];
        let hhs: Vec<Household> = s.households.values().filter(|h| !h.dissolved && !h.members.is_empty()).cloned().collect();
        for h in &hhs {
            let reserve = household_need(s, h) * 30 * gp / 1000;
            let spare = s.acct(&h.id, "coin") - reserve;
            if spare > 0 {
                let spend = spare * 15 / 1000;
                let _ = s.transfer(&h.id, "abroad", "coin", spend * 80 / 100);
                let _ = s.transfer(&h.id, "org_compact", "coin", spend * 12 / 100);
                let _ = s.transfer(&h.id, "org_lodge", "coin", spend * 8 / 100);
            }
        }
        let hc = s.acct("hinterland", "coin") - 4000 * M;
        if hc > 0 {
            let _ = s.transfer("hinterland", "abroad", "coin", hc * 2 / 100);
        }
    }
    // Compact dividend to its sworn factors
    if day % 7 == 0 && s.acct("org_compact", "coin") > 9000 * M {
        let pot = (s.acct("org_compact", "coin") - 9000 * M) / 10;
        let factors: Vec<Id> = s.orgs["org_compact"].members.iter().filter(|m| s.persons.get(*m).map(|p| p.alive() && p.occupation == "factor").unwrap_or(false)).map(|m| s.persons[m].household.clone()).collect();
        if !factors.is_empty() {
            for f in &factors {
                let _ = s.transfer("org_compact", f, "coin", pot / factors.len() as i64);
            }
        }
    }
    // the Lodge's widows-and-orphans fund: surplus is shared among members
    if day % 7 == 0 && s.acct("org_lodge", "coin") > 5000 * M {
        let pot = (s.acct("org_lodge", "coin") - 5000 * M) / 8;
        let hh: Vec<Id> = s.orgs["org_lodge"].members.iter().filter_map(|m| s.persons.get(m)).filter(|p| p.alive()).map(|p| p.household.clone()).collect();
        for h in &hh {
            let _ = s.transfer("org_lodge", h, "coin", pot / hh.len() as i64);
        }
    }
    // Hall relief from the town reserve
    if s.orgs["org_reeve"].policy.get("relief").copied().unwrap_or(0) > 0 && s.acct("org_reeve", "grain") > 0 {
        let hhs: Vec<Household> = s.households.values().filter(|h| !h.dissolved && !h.members.is_empty()).cloned().collect();
        let mut given = 0;
        for h in &hhs {
            let hunger: i64 = h.members.iter().filter_map(|m| s.persons.get(m)).map(|p| p.hunger).sum::<i64>() / h.members.len().max(1) as i64;
            if hunger > 30 {
                given += s.transfer_up_to("org_reeve", &h.id, "grain", household_need(s, h) * 2);
            }
        }
        if given > 0 && day % 7 == 0 {
            inc(out, "relief.distributed", true, format!("The Hall is handing out grain from the town reserve ({} measures today).", given / M), json!({"grain_milli": given}));
        }
    }
}

pub fn avg_hunger(s: &WorldState) -> i64 {
    let (sum, n) = s.living().filter(|p| !p.is_pc).fold((0, 0), |(a, n), p| (a + p.hunger, n + 1));
    if n == 0 {
        0
    } else {
        sum / n
    }
}

fn record_stats(s: &mut WorldState, volume: BTreeMap<String, i64>, crimes: i64) {
    let coin_households: i64 = s.households.keys().map(|h| s.acct(h, "coin")).sum();
    let grain_stock: i64 = s.accounts.iter().filter(|(k, _)| k.ends_with("|grain") && !k.starts_with("abroad") && !k.starts_with("hinterland")).map(|(_, v)| *v).sum();
    let stat = DayStat { day: s.meta.day, prices: s.market.prices.clone(), volume, population: s.population(), hunger: avg_hunger(s), crimes, coin_households, grain_stock };
    s.market.history.push(stat);
    if s.market.history.len() > 720 {
        s.market.history.remove(0);
    }
}
