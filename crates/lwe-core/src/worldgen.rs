//! Deterministic genesis of the first vertical slice (Decisions 34, 42):
//! Marrowick, a river trading town of roughly 250 persistent residents with
//! households, three generations of genealogy, three competing organisations,
//! a hinterland cohort (tier 0) and a seeded world bible.
//!
//! Everything authored here is stamped `authored_fixture`; everything derived
//! from the seed is `generated`. Nothing is copied from any published setting.

use crate::model::*;
use crate::rng::Rng;
use std::collections::BTreeMap;

const MALE: [&str; 30] = [
    "Aldren", "Bram", "Corvin", "Dain", "Edrik", "Fenn", "Garrow", "Hale", "Ivo", "Joss", "Kellan", "Lorn", "Maddoc",
    "Nial", "Oswin", "Perrin", "Quill", "Rurik", "Stellan", "Tobin", "Ulf", "Varro", "Wend", "Yorick", "Zev", "Arlo",
    "Brannoc", "Cade", "Emrys", "Teague",
];
const FEMALE: [&str; 30] = [
    "Ansa", "Brenna", "Cael", "Dessa", "Elowen", "Fira", "Gwen", "Hesper", "Isolde", "Junia", "Kerra", "Liss", "Maren",
    "Nessa", "Orla", "Petra", "Quenna", "Rhosyn", "Sunniva", "Tamsin", "Una", "Vesna", "Wynne", "Ysolt", "Zara",
    "Avice", "Bryn", "Corisande", "Edda", "Thea",
];
const FAM_A: [&str; 22] = [
    "Ash", "Black", "Cold", "Dun", "Eel", "Fen", "Gray", "Hart", "Iron", "Kettle", "Long", "Marsh", "Nether", "Oak",
    "Pike", "Reed", "Salt", "Thorn", "Under", "Weir", "Wold", "Yar",
];
const FAM_B: [&str; 12] =
    ["wick", "ford", "mere", "holt", "by", "croft", "water", "well", "barrow", "lock", "stone", "field"];
const LOOKS: [&str; 12] = [
    "weathered hands and a river-squint", "a broken nose set crooked", "ink-stained fingers", "a braid bound in eel-skin",
    "a slow, careful way of speaking", "a laugh that carries across the quay", "a limp from an old barge accident",
    "pale eyes and a watchful stillness", "flour or chaff forever in the hair", "a scar across one eyebrow",
    "rings of river-glass on every finger", "a voice hoarse from calling across water",
];

fn loc(id: &str, name: &str, kind: &str, x: i64, y: i64, residential: bool, desc: &str) -> Location {
    Location { id: id.into(), name: name.into(), kind: kind.into(), x, y, desc: desc.into(), links: vec![], residential }
}

fn locations() -> BTreeMap<Id, Location> {
    let mut v = vec![
        loc("loc_bridge", "Weir Bridge", "bridge", 500, 350, false, "Seven stone arches over the Sable, older than the town. The toll-booth is newer and resented."),
        loc("loc_market", "Saltmarket Square", "market", 470, 240, false, "Trestle stalls under oiled canvas; prices chalked on slates and argued over daily."),
        loc("loc_hall", "Reeve's Hall", "civic", 370, 170, false, "A timber hall with a stone undercroft where the tally-rolls and the town seal are kept."),
        loc("loc_gaol", "The Stone Cellar", "gaol", 320, 120, false, "The undercroft gaol. Cold in every season."),
        loc("loc_granary", "Compact Granary", "warehouse", 610, 260, false, "A raised granary on staddle stones with a single iron-bound door facing the quay."),
        loc("loc_quay", "Long Quay", "quay", 660, 330, false, "Barges moor three abreast when the river allows. Cranes, rope, and the smell of tar."),
        loc("loc_lodge", "Ferrymen's Lodge", "guildhall", 760, 300, false, "A boathouse-hall hung with oars carved with the names of the drowned."),
        loc("loc_tavern", "The Drowned Lantern", "tavern", 540, 440, false, "Low beams, a long hearth, and a lantern salvaged from a wreck that nobody admits to lighting."),
        loc("loc_shrine", "Shrine of the Patient Water", "shrine", 410, 480, false, "A spring-fed basin under a slate roof. Offerings are small, practical, and frequent."),
        loc("loc_smithy", "Anvil Row", "district", 290, 270, true, "Forges and cooperages; the street is black with clinker."),
        loc("loc_weavers", "Weavers' Lane", "district", 590, 150, true, "Tall narrow houses with loom-lofts and dye-stained gutters."),
        loc("loc_strand", "Fishers' Strand", "district", 790, 450, true, "Net-sheds, smoke-racks and upturned skiffs along the south bank."),
        loc("loc_fields", "Upper Fields", "farmland", 170, 110, true, "Strip fields and farmsteads on the terrace above the flood line."),
        loc("loc_timber", "Timber Yard", "district", 850, 190, true, "Stacked oak and alder floated down from the Wold; saw-pits and charcoal clamps."),
        loc("loc_southgate", "Southgate Rows", "district", 430, 580, true, "Close-built rows by the south gate where labourers and newcomers lodge."),
        loc("loc_oldweir", "The Old Weir", "ruin", 140, 370, false, "A collapsed weir of fitted black stone upstream. The water is too still there, and the fish avoid it."),
    ];
    let links: [(&str, &str, i64); 22] = [
        ("loc_bridge", "loc_market", 5), ("loc_bridge", "loc_tavern", 4), ("loc_bridge", "loc_quay", 6),
        ("loc_market", "loc_hall", 5), ("loc_market", "loc_granary", 5), ("loc_market", "loc_smithy", 7),
        ("loc_market", "loc_weavers", 6), ("loc_hall", "loc_gaol", 2), ("loc_hall", "loc_fields", 12),
        ("loc_granary", "loc_quay", 3), ("loc_quay", "loc_lodge", 4), ("loc_lodge", "loc_timber", 8),
        ("loc_weavers", "loc_timber", 10), ("loc_tavern", "loc_shrine", 5), ("loc_tavern", "loc_strand", 9),
        ("loc_tavern", "loc_southgate", 6), ("loc_shrine", "loc_southgate", 4), ("loc_smithy", "loc_fields", 10),
        ("loc_smithy", "loc_oldweir", 14), ("loc_fields", "loc_oldweir", 15), ("loc_strand", "loc_lodge", 9),
        ("loc_shrine", "loc_oldweir", 18),
    ];
    for (a, b, m) in links {
        for l in v.iter_mut() {
            if l.id == a {
                l.links.push((b.to_string(), m));
            } else if l.id == b {
                l.links.push((a.to_string(), m));
            }
        }
    }
    v.into_iter().map(|l| (l.id.clone(), l)).collect()
}

fn entry(title: &str, body: &str) -> BibleEntry {
    BibleEntry { title: title.into(), body: body.into(), provenance: "authored_fixture".into() }
}

fn bible() -> BTreeMap<String, Vec<BibleEntry>> {
    let mut b = BTreeMap::new();
    b.insert("01 Cosmology".into(), vec![
        entry("The Lattice", "Reality is a stack of 'leaves' — self-consistent worlds with their own local constants — bound along a shared mathematical spine called the Lattice. Each leaf has a law contract (see Magic). Marrowick stands on the leaf its scholars call the Middle Water. Adjacent leaves are not 'heavens' or 'hells'; they are simply elsewhere, with different arithmetic."),
        entry("The Underweir (adjacent leaf)", "Where the potential field thins — still water, deep stone, places of long grief — the neighbouring leaf called the Underweir presses close. Its time runs slower by a measured ratio (river-scholars claim 7:1). Nothing from it has been proved to cross. The Old Weir is the thinnest place within a day's walk."),
        entry("Layered multiverse (status)", "Only the Middle Water is simulated in this slice. Other leaves, interstellar civilisations and time travel are specified (Decisions 08, 54) but not implemented; entries about them are lore hypotheses, not simulated fact."),
    ]);
    b.insert("02 Astronomy & Calendar".into(), vec![
        entry("Year and sky", "The year has 360 days in twelve months of thirty: Thawmonth, Seedmonth, Greenmonth, Highsun, Goldmonth, Reapmonth, Fallmonth, Mistmonth, Frostmonth, Deepwinter, Icemonth, Stirring. Two moons: pale Sister (30-day cycle, governs the market calendar) and small red Ember (retrograde, 11 days), whose conjunctions the superstitious call 'tally nights'."),
        entry("Reckoning", "Years are counted 'of the Weir' from the building of Weir Bridge. The simulation begins on 1 Thawmonth 412."),
    ]);
    b.insert("03 Geology & Geography".into(), vec![
        entry("The Sable valley", "The River Sable cuts a broad alluvial valley through folded limestone and black mudstone. The town sits on a gravel terrace at the lowest fordable point. Upstream the Wold is karst: sinkholes, caves, disappearing streams. Downstream the river widens into reed fen before the salt estuary, eleven days by barge."),
        entry("Resources", "Alluvial loam on the terraces (grain), oak and alder on the Wold (timber, charcoal), bog iron in the fen margins, fuller's earth near Weavers' Lane, and river-glass — a dark vitreous stone found only near the Old Weir — which holds magical potential unusually well."),
        entry("Hazards", "Spring floods when Wold snowmelt meets rain; summer droughts that strand barges; winter ice some years. The granary stands on staddle stones for a reason."),
    ]);
    b.insert("04 Climate, Flora & Fauna".into(), vec![
        entry("Climate", "Temperate and wet, with a pronounced growing season from Seedmonth through Goldmonth and harvest in Reapmonth. Crop yield in the simulation is a function of rainfall during the growing season: too little or too much both reduce the growth index."),
        entry("Flora", "Sable barley (the staple grain), marsh-flax (cloth), black alder, river oak, sedge and bulrush, lantern-moss (faintly luminous where potential pools), and wolfsbane-of-the-fen, which is poisonous and sold anyway."),
        entry("Fauna", "Sable pike, silver eel, river lamprey; otter, heron, marsh harrier; wild boar and fallow deer on the Wold; wolves in hard winters. The 'weir-eel' of local story is unclassified: fishers swear to it, no scholar has a specimen."),
    ]);
    b.insert("05 Peoples & Languages".into(), vec![
        entry("Peoples", "The valley folk are of one broad ancestry with river, Wold and fen sub-cultures distinguished by trade, dialect and custom rather than by nature. Ancestry carries no moral alignment (Decision 47)."),
        entry("Languages", "Sabline is the common tongue (registers: quay-cant, hall-speech). Old Wold survives in oaths, lullabies and place-names; few can read its knot-script. Fen-traders use Brackish, a contact pidgin. Not every resident speaks every tongue: proficiency is recorded per person."),
    ]);
    b.insert("06 Society, Politics & Trade".into(), vec![
        entry("Three powers", "The Reeve's Hall holds the charter, the seal, the watch and the right to tax. The Granary Compact — a league of grain-factors — owns the granary, the barges and most of the credit. The Ferrymen's Lodge controls every crossing and every pilot who knows the river's moods. None can rule without the other two, and each remembers every slight."),
        entry("Economy", "Coin is the silver mark. Households produce, earn, eat and trade daily in the Saltmarket; prices move with supply and demand; the Compact arbitrages against downstream prices when barges can run; the Hall taxes sales and hearths; transit trade brings coin in and imported wares take it out. Every unit of coin and grain is ledgered and conserved."),
        entry("Law", "Theft is punished by fine and the Stone Cellar; assault likewise; the watch-captain investigates on evidence and testimony — and testimony can be mistaken or malicious."),
    ]);
    b.insert("07 Magic (law contract mw-1)".into(), vec![
        entry("Potential", "Magic is the movement of a conserved quantity called potential, measured in thaums. On the Middle Water the total is fixed: ambient + bound + dissipated = constant. A working draws ambient potential, binds part of it into an effect and dissipates the rest as cold, noise or light. Dissipated potential relaxes back to ambient at a slow fixed rate. Nothing creates potential except an explicit, logged opening to another leaf."),
        entry("Cost law", "Cost(level) = 4 × level² thaums, of which efficiency η (30–70% by training) is bound and (1−η) dissipated. A working that would draw more than the ambient field holds simply fails: there is no overdraft. D&D spell slots are the practitioner's personal capacity to shape potential; the ambient field is the world's capacity to supply it."),
        entry("Traditions", "Hedge-practitioners speak of 'asking the water'; Hall scholars keep tables; the Lodge's pilots use it without naming it. All are describing the same ledger with different accuracy."),
    ]);
    b.insert("08 Arts, Music & Cuisine".into(), vec![
        entry("Music", "Instruments follow materials: alder flutes, eel-skin frame drums, reed pipes, and the 'sable harp' strung with waxed flax. Scales are pentatonic with a characteristic flattened fourth; work-songs are call-and-response to the stroke of an oar."),
        entry("Art & architecture", "Timber-framed houses on stone footings above flood height; carved oar-blades as memorials; dye-work in madder and weld; river-glass set in lead as charms. Building follows materials, climate and the flood line, not fashion."),
        entry("Cuisine", "Barley bread and pottage, smoked eel, pike in sorrel, small ale with every meal, honey cakes at harvest. In lean years: reed-root flour and resentment."),
    ]);
    b.insert("09 Science & Scholarship".into(), vec![
        entry("Natural philosophy", "The Hall keeps river-level tallies going back ninety years and can predict floods better than it admits. A healer's craft distinguishes wound-fever from river-ague. Lens-grinding has reached the valley; clockwork has not. Knowledge spreads person to person and can be lost."),
    ]);
    b.insert("10 Dread & Mystery".into(), vec![
        entry("The Old Weir", "The black stone of the Old Weir is fitted without mortar and is older than any record. Fish avoid the pool. Lantern-moss grows thick. On tally nights some residents dream the same dream of counting. This is a seeded mystery: the simulation tracks potential at the site; whether anything answers is for play to discover."),
    ]);
    b
}

struct Proto {
    name: String,
    family: String,
    sex: String,
    age: i64,
    clan: usize,
    parents: Vec<usize>,
    spouse: Option<usize>,
    alive: bool,
    children: Vec<usize>,
}

fn clamp(v: i64, lo: i64, hi: i64) -> i64 {
    v.max(lo).min(hi)
}

/// Set a symmetric pair of relation edges.
pub fn bond(s: &mut WorldState, a: &str, b: &str, kind: &str, fond: i64, trust: i64) {
    for (x, y) in [(a, b), (b, a)] {
        let r = s.rel_mut(x, y);
        r.fondness = r.fondness.max(fond);
        r.trust = r.trust.max(trust);
        r.respect = r.respect.max(45);
        r.familiarity = r.familiarity.max(fond);
        if !r.kinds.iter().any(|k| k == kind) {
            r.kinds.push(kind.to_string());
        }
        if r.affinity() >= 55 && r.status == "neutral" {
            r.status = "friend".into();
        }
    }
}

pub fn stat_block(rng: &mut Rng, occupation: &str) -> Stats {
    let mut ab = [0i64; 6];
    for a in ab.iter_mut() {
        *a = clamp(rng.bell(10, 5), 6, 16);
    }
    let con = (ab[2] - 10).div_euclid(2);
    let (class, hp, ac, atk, dd, ds, db) = match occupation {
        "guard" => ("guard", 11 + con, 16, 3, 1, 6, 1),
        "smith" | "woodcutter" | "porter" | "ferryman" | "labourer" => ("tough", 8 + con, 11, 3, 1, 6, 1),
        "mage" => ("hedge-mage", 6 + con, 11, 4, 1, 8, 0),
        _ => ("commoner", 4 + con, 10, 2, 1, 4, 0),
    };
    let hp = hp.max(2);
    Stats { abilities: ab, level: 1, class: class.into(), hp, max_hp: hp, ac, attack_bonus: atk, dmg_dice: dd, dmg_sides: ds, dmg_bonus: db, exhaustion: 0, potions: 0 }
}

pub fn random_traits(rng: &mut Rng) -> Traits {
    let mut t = || clamp(rng.bell(50, 45), 5, 95);
    Traits { ambition: t(), empathy: t(), courage: t(), honesty: t(), sociability: t(), temper: t(), piety: t(), curiosity: t() }
}

pub fn home_for(rng: &mut Rng, occ: &str) -> &'static str {
    match occ {
        "farmer" => "loc_fields",
        "fisher" | "ferryman" => "loc_strand",
        "woodcutter" => "loc_timber",
        "smith" => "loc_smithy",
        "weaver" | "factor" | "reeve" | "clerk" => "loc_weavers",
        _ => *rng.pick(&["loc_southgate", "loc_southgate", "loc_smithy", "loc_weavers", "loc_strand"]),
    }
}
pub fn work_for(occ: &str, home: &str) -> String {
    match occ {
        "farmer" => "loc_fields",
        "fisher" => "loc_strand",
        "ferryman" | "labourer" => "loc_quay",
        "porter" => "loc_granary",
        "factor" => "loc_market",
        "weaver" => "loc_weavers",
        "brewer" | "innkeeper" => "loc_tavern",
        "smith" => "loc_smithy",
        "woodcutter" => "loc_timber",
        "guard" | "clerk" | "reeve" => "loc_hall",
        "healer" | "priest" => "loc_shrine",
        "mage" => "loc_oldweir",
        _ => home,
    }
    .to_string()
}
pub fn employer_for(occ: &str) -> Option<Id> {
    match occ {
        "porter" | "factor" => Some("org_compact".into()),
        "ferryman" => Some("org_lodge".into()),
        "guard" | "clerk" | "reeve" => Some("org_reeve".into()),
        _ => None,
    }
}

pub fn generate(world_id: &str, name: &str, seed: u64) -> WorldState {
    let mut s = WorldState::default();
    s.meta = Meta { world_id: world_id.into(), name: name.into(), seed, engine: crate::ENGINE_VERSION.into(), day: 0, town: "Marrowick".into() };
    s.locations = locations();
    s.bible = bible();
    s.hinterland_pop = 900;
    let mut rng = Rng::new(seed, "genesis", 0, 0);

    // --- three generations of people -------------------------------------------------
    let mut protos: Vec<Proto> = vec![];
    let mut families: Vec<String> = vec![];
    let mut adults: Vec<usize> = vec![];
    let mut clan = 0usize;
    while adults.len() < 118 {
        let fam = loop {
            let f = format!("{}{}", rng.pick(&FAM_A), rng.pick(&FAM_B));
            if !families.contains(&f) {
                break f;
            }
        };
        families.push(fam.clone());
        let f_age = rng.range(58, 78);
        let m_age = f_age - rng.range(0, 6);
        let fi = protos.len();
        protos.push(Proto { name: rng.pick(&MALE).to_string(), family: fam.clone(), sex: "m".into(), age: f_age, clan, parents: vec![], spouse: Some(fi + 1), alive: f_age < 72 && rng.chance(380), children: vec![] });
        protos.push(Proto { name: rng.pick(&FEMALE).to_string(), family: fam.clone(), sex: "f".into(), age: m_age, clan, parents: vec![], spouse: Some(fi), alive: m_age < 74 && rng.chance(450), children: vec![] });
        let n = *rng.pick(&[1, 2, 2, 3, 3, 3, 4]);
        for _ in 0..n {
            let sex = if rng.chance(500) { "m" } else { "f" };
            let age = clamp(m_age - rng.range(19, 36), 18, 50);
            let idx = protos.len();
            let nm = if sex == "m" { rng.pick(&MALE) } else { rng.pick(&FEMALE) };
            protos.push(Proto { name: nm.to_string(), family: fam.clone(), sex: sex.into(), age, clan, parents: vec![fi, fi + 1], spouse: None, alive: true, children: vec![] });
            protos[fi].children.push(idx);
            protos[fi + 1].children.push(idx);
            adults.push(idx);
        }
        clan += 1;
    }
    // marriages across clans
    let mut men: Vec<usize> = adults.iter().copied().filter(|i| protos[*i].sex == "m").collect();
    let mut women: Vec<usize> = adults.iter().copied().filter(|i| protos[*i].sex == "f").collect();
    for v in [&mut men, &mut women] {
        for i in (1..v.len()).rev() {
            let j = rng.below(i as u64 + 1) as usize;
            v.swap(i, j);
        }
    }
    let mut used_w = vec![false; women.len()];
    for &m in &men {
        if !rng.chance(820) {
            continue;
        }
        for (wi, &w) in women.iter().enumerate() {
            if used_w[wi] || protos[w].clan == protos[m].clan || (protos[w].age - protos[m].age).abs() > 9 {
                continue;
            }
            used_w[wi] = true;
            protos[m].spouse = Some(w);
            protos[w].spouse = Some(m);
            break;
        }
    }
    // children of the current generation
    for &m in &men {
        let Some(w) = protos[m].spouse else { continue };
        let years = protos[m].age.min(protos[w].age) - 19;
        if years <= 0 {
            continue;
        }
        let k = rng.range(0, (years / 3).min(4));
        for _ in 0..k {
            let sex = if rng.chance(500) { "m" } else { "f" };
            let age = rng.range(0, years.min(17));
            let idx = protos.len();
            let nm = if sex == "m" { rng.pick(&MALE) } else { rng.pick(&FEMALE) };
            let fam = protos[m].family.clone();
            protos.push(Proto { name: nm.to_string(), family: fam, sex: sex.into(), age, clan: protos[m].clan, parents: vec![m, w], spouse: None, alive: true, children: vec![] });
            protos[m].children.push(idx);
            protos[w].children.push(idx);
        }
    }

    // --- occupations -----------------------------------------------------------------
    let quotas: [(&str, usize); 16] = [
        ("reeve", 1), ("clerk", 3), ("guard", 6), ("factor", 5), ("porter", 8), ("ferryman", 10), ("innkeeper", 2),
        ("healer", 1), ("priest", 1), ("mage", 1), ("farmer", 14), ("fisher", 14), ("weaver", 9), ("brewer", 4),
        ("smith", 4), ("woodcutter", 6),
    ];
    let mut order = adults.clone();
    for i in (1..order.len()).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        order.swap(i, j);
    }
    let mut occ: BTreeMap<usize, String> = BTreeMap::new();
    let mut it = order.iter();
    for (o, n) in quotas {
        for _ in 0..n {
            if let Some(i) = it.next() {
                occ.insert(*i, o.to_string());
            }
        }
    }
    for i in it {
        occ.insert(*i, rng.pick(&["labourer", "labourer", "farmer", "fisher", "weaver"]).to_string());
    }

    // --- materialise persons -----------------------------------------------------------
    let ids: Vec<Id> = (0..protos.len()).map(|i| format!("p{:04}", i + 1)).collect();
    for (i, p) in protos.iter().enumerate() {
        let occupation = if p.age >= 65 {
            "elder".to_string()
        } else if p.age < 15 {
            "child".to_string()
        } else {
            occ.get(&i).cloned().unwrap_or_else(|| "labourer".into())
        };
        let mut prng = Rng::new(seed, "person", i as u64, 0);
        let stats = stat_block(&mut prng, &occupation);
        let mut languages = vec!["Sabline".to_string()];
        if prng.chance(150) {
            languages.push("Old Wold".into());
        }
        if matches!(occupation.as_str(), "factor" | "ferryman" | "porter") && prng.chance(500) {
            languages.push("Brackish".into());
        }
        let person = Person {
            id: ids[i].clone(),
            name: p.name.clone(),
            family: p.family.clone(),
            sex: p.sex.clone(),
            birth_day: -(p.age * DAYS_PER_YEAR + prng.range(0, 359)),
            status: if p.alive { "alive".into() } else { "dead".into() },
            death_day: if p.alive { None } else { Some(-prng.range(100, 3000)) },
            occupation: occupation.clone(),
            employer: employer_for(&occupation),
            tier: 1,
            traits: random_traits(&mut prng),
            health: clamp(prng.bell(85, 20) - (p.age - 50).max(0), 30, 100),
            hunger: 5,
            parents: p.parents.iter().map(|x| ids[*x].clone()).collect(),
            spouse: p.spouse.map(|x| ids[x].clone()),
            children: p.children.iter().map(|x| ids[*x].clone()).collect(),
            stats,
            agent_profile: "t1-resident".into(),
            appearance: prng.pick(&LOOKS).to_string(),
            languages,
            ..Default::default()
        };
        s.persons.insert(person.id.clone(), person);
    }
    s.counters.insert("p".into(), protos.len() as u64);

    // --- households ------------------------------------------------------------------
    let mut hh_of: BTreeMap<usize, Id> = BTreeMap::new();
    let new_hh = |s: &mut WorldState, head: usize, rng: &mut Rng, occ: &str| -> Id {
        let id = s.next_id("h");
        let home = home_for(rng, occ).to_string();
        s.households.insert(id.clone(), Household { id: id.clone(), name: format!("House {}", protos[head].family), head: ids[head].clone(), members: vec![], home, dissolved: false, last_gouge_grievance: -1000 });
        id
    };
    for &a in &adults {
        if hh_of.contains_key(&a) {
            continue;
        }
        let o = occ.get(&a).cloned().unwrap_or_default();
        match protos[a].spouse {
            Some(sp) => {
                let head = if protos[a].sex == "m" { a } else { sp };
                let ho = occ.get(&head).cloned().unwrap_or_default();
                let id = new_hh(&mut s, head, &mut rng, &ho);
                hh_of.insert(a, id.clone());
                hh_of.insert(sp, id);
            }
            None => {
                let id = new_hh(&mut s, a, &mut rng, &o);
                hh_of.insert(a, id);
            }
        }
    }
    for i in 0..protos.len() {
        if hh_of.contains_key(&i) || !protos[i].alive {
            continue;
        }
        // children live with parents; living elders with their first child
        let host = if !protos[i].parents.is_empty() { hh_of.get(&protos[i].parents[0]).cloned() } else { protos[i].children.first().and_then(|c| hh_of.get(c).cloned()) };
        if let Some(h) = host {
            hh_of.insert(i, h);
        }
    }
    for (i, h) in &hh_of {
        if !protos[*i].alive {
            continue;
        }
        let home = s.households[h].home.clone();
        s.households.get_mut(h).unwrap().members.push(ids[*i].clone());
        let p = s.persons.get_mut(&ids[*i]).unwrap();
        p.household = h.clone();
        p.home = home.clone();
        p.work = work_for(&p.occupation, &home);
    }

    // --- organisations ---------------------------------------------------------------
    let mk_org = |id: &str, name: &str, kind: &str, charter: &str, seat: &str, policy: &[(&str, i64)], goals: &[&str]| Org {
        id: id.into(), name: name.into(), kind: kind.into(), charter: charter.into(), leader: None, members: vec![], seat: seat.into(),
        policy: policy.iter().map(|(k, v)| (k.to_string(), *v)).collect(), goals: goals.iter().map(|g| g.to_string()).collect(), unpaid_days: 0,
    };
    s.orgs.insert("org_compact".into(), mk_org("org_compact", "The Granary Compact", "merchant league",
        "A sworn league of grain-factors holding the granary and barges in common; profit is divided by share, losses by blame.",
        "loc_granary", &[("hoarding", 0), ("wage_milli", 7000), ("factor_wage_milli", 12000), ("barge_capacity", 160 * M)],
        &["Control the valley's grain trade", "Keep the Hall's taxes off the quay", "Break the Lodge's hold on freight fees"]));
    s.orgs.insert("org_lodge".into(), mk_org("org_lodge", "The Ferrymen's Lodge", "guild",
        "Every pilot, ferryman and lighterman on the Sable swears to the Lodge: one fee, one river, no scabs.",
        "loc_lodge", &[("ferry_fee_milli", 150), ("wage_milli", 8000), ("strike", 0)],
        &["Keep the crossing a Lodge monopoly", "Raise freight fees without losing trade", "See drowned members' families fed"]));
    s.orgs.insert("org_reeve".into(), mk_org("org_reeve", "The Reeve's Hall", "town government",
        "Charter-holder of Marrowick: keeps the peace, the seal, the tally-rolls and the reserve granary; levies market and hearth dues.",
        "loc_hall", &[("tax_permille", 80), ("hearth_tax_milli", 15000), ("wage_milli", 8500), ("edict_antihoarding", 0), ("relief", 0)],
        &["Keep the town fed and quiet", "Balance the tally-rolls", "Stop either guild from owning the Hall"]));
    for p in s.persons.values() {
        if let (true, Some(e)) = (p.alive(), &p.employer) {
            s.orgs.get_mut(e).unwrap().members.push(p.id.clone());
        }
    }
    for (org, role) in [("org_compact", "factor"), ("org_lodge", "ferryman"), ("org_reeve", "reeve")] {
        let leader = s.orgs[org].members.iter().filter(|m| s.persons[*m].occupation == role).max_by_key(|m| (s.persons[*m].traits.ambition, (*m).clone())).cloned();
        s.orgs.get_mut(org).unwrap().leader = leader.clone();
        if let Some(l) = leader {
            let p = s.persons.get_mut(&l).unwrap();
            p.tier = 3;
            p.agent_profile = "t3-major".into();
            p.flags.push(format!("leader:{org}"));
            p.stats.level = 3;
            p.stats.max_hp += 10;
            p.stats.hp = p.stats.max_hp;
        }
    }
    // Watch-captain: the wisest guard. Leads investigations.
    if let Some(cap) = s.persons.values().filter(|p| p.occupation == "guard").max_by_key(|p| (p.stats.abilities[4], p.id.clone())).map(|p| p.id.clone()) {
        let p = s.persons.get_mut(&cap).unwrap();
        p.flags.push("watch_captain".into());
        p.tier = 3;
        p.agent_profile = "t3-major".into();
        p.stats.level = 3;
        p.stats.max_hp += 12;
        p.stats.hp = p.stats.max_hp;
        p.stats.attack_bonus = 5;
    }
    // The world-shaping actor (T4): the most ambitious, least scrupulous factor who is not the Compact's leader.
    let leader = s.orgs["org_compact"].leader.clone();
    if let Some(v) = s.persons.values().filter(|p| p.occupation == "factor" && Some(&p.id) != leader.as_ref()).max_by_key(|p| (p.traits.ambition - p.traits.honesty, p.id.clone())).map(|p| p.id.clone()) {
        let p = s.persons.get_mut(&v).unwrap();
        p.tier = 4;
        p.agent_profile = "custom-schemer".into();
        p.flags.push("schemer".into());
        p.traits.ambition = p.traits.ambition.max(90);
        p.traits.honesty = p.traits.honesty.min(25);
        p.traits.courage = p.traits.courage.max(60);
        p.goals = vec!["Take the Compact from within".into(), "Own the town's debts before the next lean year".into(), "Never be seen holding the knife".into()];
        p.stats.level = 4;
        p.stats.max_hp += 14;
        p.stats.hp = p.stats.max_hp;
    }
    for (a, b, sc) in [("org_compact", "org_lodge", -12), ("org_compact", "org_reeve", -5), ("org_lodge", "org_reeve", 8)] {
        s.org_relations.insert(org_key(a, b), OrgRelation { score: sc, status: "neutral".into(), history: vec![] });
    }

    // --- assets ------------------------------------------------------------------------
    let innkeeper_hh = s.persons.values().find(|p| p.occupation == "innkeeper").map(|p| p.household.clone()).unwrap_or_else(|| "org_reeve".into());
    for (id, name, kind, owner, l, prot) in [
        ("asset_granary", "Compact Granary", "warehouse", "org_compact", "loc_granary", false),
        ("asset_barge_heron", "Barge 'Grey Heron'", "barge", "org_compact", "loc_quay", false),
        ("asset_barge_otter", "Barge 'Otter's Due'", "barge", "org_compact", "loc_quay", false),
        ("asset_ferry", "The Chain Ferry", "ferry", "org_lodge", "loc_lodge", false),
        ("asset_hall", "Reeve's Hall", "hall", "org_reeve", "loc_hall", true),
        ("asset_reserve", "Town Reserve Granary", "warehouse", "org_reeve", "loc_hall", false),
        ("asset_tavern", "The Drowned Lantern", "tavern", innkeeper_hh.as_str(), "loc_tavern", false),
        ("asset_bridge", "Weir Bridge", "bridge", "org_reeve", "loc_bridge", true),
    ] {
        s.assets.insert(id.into(), Asset { id: id.into(), name: name.into(), kind: kind.into(), owner: owner.into(), location: l.into(), active: true, condition: 100, protected: prot });
    }

    // --- relations -----------------------------------------------------------------------
    let pids: Vec<Id> = s.persons.keys().cloned().collect();
    for id in &pids {
        let p = s.persons[id].clone();
        if !p.alive() {
            continue;
        }
        if let Some(sp) = &p.spouse {
            if s.persons[sp].alive() {
                bond(&mut s, id, sp, "spouse", 78, 80);
            }
        }
        for par in &p.parents {
            if s.persons[par].alive() {
                bond(&mut s, id, par, "kin", 72, 76);
            }
            for sib in s.persons[par].children.clone() {
                if &sib != id && s.persons[&sib].alive() {
                    bond(&mut s, id, &sib, "kin", 62, 64);
                }
            }
        }
    }
    let workers: Vec<Person> = s.living().filter(|p| p.age(0) >= 15).cloned().collect();
    for p in &workers {
        let mut prng = Rng::new(seed, "bond", crate::rng::hash_str(&p.id), 0);
        let mates: Vec<&Person> = workers.iter().filter(|q| q.id != p.id && (q.work == p.work || q.home == p.home)).collect();
        for _ in 0..4 {
            if mates.is_empty() {
                break;
            }
            let q = *prng.pick(&mates);
            let kind = if q.work == p.work { "coworker" } else { "neighbour" };
            let f = clamp(prng.bell(48, 30), 15, 85);
            bond(&mut s, &p.id, &q.id, kind, f, clamp(f + prng.range(-10, 10), 10, 90));
        }
    }

    // --- opening ledger (explicit issuance; conserved thereafter) ---------------------
    for (g, p) in [("grain", 2000), ("fish", 2500), ("ale", 3000), ("cloth", 30000), ("timber", 1500), ("tools", 40000)] {
        s.market.prices.insert(g.into(), p);
        s.market.base.insert(g.into(), p);
        s.market.foreign.insert(g.into(), p);
    }
    let hh_ids: Vec<Id> = s.households.keys().cloned().collect();
    for h in &hh_ids {
        let hh = s.households[h].clone();
        let need: i64 = hh.members.iter().map(|m| crate::sim::food_need(&s.persons[m], 0)).sum();
        let adults_n = hh.members.iter().filter(|m| s.persons[*m].age(0) >= 15).count() as i64;
        let head_occ = s.persons.get(&hh.head).map(|p| p.occupation.clone()).unwrap_or_default();
        s.issue(h, "coin", (rng.range(70, 190) * adults_n.max(1)) * M);
        s.issue(h, "grain", need * 6);
        s.issue(h, "timber", 12 * M);
        if hh.members.iter().any(|m| s.persons[m].occupation == "farmer") {
            s.issue(h, "grain", need * 150 + 350 * M);
        }
        if head_occ == "innkeeper" || head_occ == "brewer" {
            s.issue(h, "ale", 60 * M);
            s.issue(h, "grain", 30 * M);
        }
    }
    s.issue("org_compact", "coin", 7000 * M);
    s.issue("org_compact", "grain", 4500 * M);
    s.issue("org_lodge", "coin", 3000 * M);
    s.issue("org_reeve", "coin", 4500 * M);
    s.issue("org_reeve", "grain", 1800 * M);
    s.issue("hinterland", "grain", 20000 * M);
    s.issue("hinterland", "coin", 3500 * M);

    // --- magic field & weather -----------------------------------------------------------
    s.magic = MagicField { law_version: "mw-1".into(), ambient: 9000 * M, bound: 0, dissipated: 1000 * M, capacity_total: 10000 * M, relax_permille: 20, spells_cast: 0 };
    s.weather = Weather { rain: 45, temp: 8, river: 55, growth_sum: 0, growth_days: 0, last_growth_index: 100, shocks: vec![], desc: "cool and showery".into() };
    s.party = Party { location: "loc_bridge".into(), minute: 8 * 60, ..Default::default() };

    // seeded lore claim everyone half-knows
    let c = s.add_claim(Claim { kind: "lore".into(), text: "Fish will not swim the pool below the Old Weir, and nobody sensible goes there on a tally night.".into(), truth: true, public: true, ..Default::default() });
    let _ = (c, pids);
    crate::sim::social::refresh_tiers(&mut s);
    s
}
