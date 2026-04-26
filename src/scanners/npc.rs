use crate::core::rng::DotNetRandom;
use std::{collections::HashMap, fs, path::PathBuf};

struct CsDict<K, V> {
    keys: Vec<K>,
    pub values: Vec<V>,
    indices: HashMap<K, usize>,
}

impl<K: Eq + std::hash::Hash + Clone, V> CsDict<K, V> {
    fn new() -> Self {
        Self {
            keys: vec![],
            values: vec![],
            indices: HashMap::new(),
        }
    }
    fn insert(&mut self, k: K, v: V) {
        if let Some(&i) = self.indices.get(&k) {
            self.values[i] = v;
        } else {
            self.indices.insert(k.clone(), self.keys.len());
            self.keys.push(k);
            self.values.push(v);
        }
    }
}

#[derive(Default)]
pub struct GameData {
    pub clothes: Vec<String>,
    pub pants: Vec<String>,
    pub weapons: Vec<String>,
    pub stuffs: Vec<String>,
    pub spells: Vec<String>,
    pub tools: Vec<String>,
    pub names_prefix: Vec<String>,
    pub names_m_suffix: Vec<String>,
    pub names_f_suffix: Vec<String>,
    pub loaded: bool,
}

impl GameData {
    pub fn load_from_dir(b: &std::path::Path) -> Self {
        let (mut im, mut sm, mut f) = (CsDict::new(), CsDict::new(), vec![]);
        fn walk(d: &std::path::Path, f: &mut Vec<PathBuf>) {
            if let Ok(es) = fs::read_dir(d) {
                for p in es.flatten().map(|e| e.path()) {
                    if p.is_dir() {
                        walk(&p, f);
                    } else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("xml") || e.eq_ignore_ascii_case("txt")) {
                        f.push(p);
                    }
                }
            }
        }
        
        ["ThingDef", "Practice/Spell", "Language"]
            .iter()
            .for_each(|d| walk(&b.join(d), &mut f));
            
        f.sort();

        let mut translations: HashMap<String, String> = HashMap::new();

        for p in &f {
            let path_str = p.to_string_lossy();
            if path_str.contains("Language") {
                if let Ok(content) = fs::read_to_string(p) {
                    if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("txt")) {
                        let lines: Vec<&str> = content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
                        for chunk in lines.chunks(2) {
                            if chunk.len() == 2 {
                                translations.insert(chunk[0].to_string(), chunk[1].to_string());
                            }
                        }
                    } else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("xml")) {
                        for line in content.lines() {
                            if let Some(start) = line.find("Name=\"") {
                                if let Some(end) = line[start + 6..].find('"') {
                                    let key = &line[start + 6..start + 6 + end];
                                    if let Some(val_start) = line.find('>') {
                                        if let Some(val_end) = line.find("</Text>") {
                                            if val_start < val_end {
                                                let val = &line[val_start + 1..val_end];
                                                translations.insert(key.trim().to_string(), val.trim().to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let translate = |s: &str| -> String {
            translations.get(s).cloned().unwrap_or_else(|| s.to_string())
        };

        let txt = |b: &str, t: &str| {
            b.split_once(&format!("<{t}>"))?
                .1
                .split_once(&format!("</{t}>"))
                .map(|(s, _)| s.trim().to_string())
        };
        
        for p in f.into_iter().filter_map(|p| fs::read_to_string(p).ok()) {
            for b in p
                .split("</ThingDef>")
                .filter_map(|s| s.split_once("<ThingDef").map(|(_, b)| b))
            {
                let n = txt(b, "defName")
                    .or_else(|| {
                        let mut r = b;
                        while let Some((left, right)) = r.split_once("Name=\"") {
                            if !left.ends_with("Parent") {
                                return right.split_once('"').map(|(s, _)| s.to_string());
                            }
                            r = right;
                        }
                        None
                    })
                    .unwrap_or_default();
                if !n.is_empty() {
                    let mut et = b
                        .split_once("Lable=\"")
                        .and_then(|(_, right)| right.split_once('"'))
                        .map(|(s, _)| s.trim().to_string())
                        .unwrap_or_default();

                    if et.is_empty() {
                        if b.contains("Parent=\"ToolBase\"") || b.contains("Parent=\"Item_Tool\"") {
                            et = "Tool".to_string();
                        }
                    }

                    let raw_tn_opt = txt(b, "ThingName");

                    if n.ends_with("Base") || (raw_tn_opt.is_none() && b.contains("Parent=\"ItemBase\"")) {
                        continue;
                    }
                    
                    let raw_tn = raw_tn_opt.unwrap_or_else(|| n.clone());
                    let tn = translate(&raw_tn);
                    if tn.is_empty() {
                        continue;
                    }
                    
                    let it = n.clone();
                    let is_stuff = ["Material", "LeftoverMaterial", "Rock", "WoodBlock", "MetalBlock", "Bone", "BambooBlock", "Stuff", "Wood", "Meat"]
                                .contains(&et.as_str())
                                || b.contains("<StuffTexPath")
                                || b.contains("<StuffCategories>");
                    im.insert(n, (tn, it, et, is_stuff));
                }
            }
            for b in p
                .split("</Spell>")
                .filter_map(|s| s.split_once("<Spell ").map(|(_, b)| b))
            {
                let n = txt(b, "defName")
                    .or_else(|| {
                        let mut r = b;
                        while let Some((left, right)) = r.split_once("Name=\"") {
                            if !left.ends_with("Parent") {
                                return right.split_once('"').map(|(s, _)| s.to_string());
                            }
                            r = right;
                        }
                        None
                    })
                    .unwrap_or_default();
                if !n.is_empty() {
                    let raw_dn = txt(b, "DisplayName")
                        .or_else(|| txt(b, "ThingName"))
                        .unwrap_or_else(|| n.clone());
                    sm.insert(n.clone(), translate(&raw_dn));
                }
            }
        }
        
        let mut d = Self::default();
        for (tn, it, et, is_stuff) in im.values {
            if et == "Clothes" {
                d.clothes.push(tn.clone());
            }
            if et == "Pants" || et == "Trousers" {
                d.pants.push(tn.clone());
            }
            if et == "Weapon" || it == "Fabao" || it.contains("Fabao") {
                d.weapons.push(tn.clone());
            }
            if et == "Tool" { 
                d.tools.push(tn.clone());
            }
            if is_stuff {
                d.stuffs.push(tn);
            }
        }
        d.spells = sm.values;
        d.loaded = !d.clothes.is_empty();
        
        let load_names = |file: &str| -> Vec<String> {
            if let Ok(content) = fs::read_to_string(b.join(file)) {
                content.split(&['\r', '\n'][..])
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_string())
                    .collect()
            } else {
                vec![]
            }
        };
        d.names_prefix = load_names("Display/NpcName/RaceName/Prefix_Human.txt");
        d.names_m_suffix = load_names("Display/NpcName/RaceName/MSuffix.txt");
        d.names_f_suffix = load_names("Display/NpcName/RaceName/FSuffix.txt");

        d
    }
}

#[derive(Debug, Clone)]
pub struct SectData {
    pub sect_name: String,
    pub elders: Vec<ElderData>,
}

#[derive(Debug, Clone)]
pub struct ElderData {
    pub name: String,
    pub level_name: String, 
    pub talismans: Vec<String>,
    pub inventory: Vec<String>,
}

/// 安全的数组访问器，杜绝 panic
fn get_safe_item(pool: &[String], index: usize) -> String {
    pool.get(index).cloned().unwrap_or_else(|| format!("未知物品_#{}", index))
}

/// 核心推演引擎：基于基础世界 Seed 透视 13 门派大能数据
pub fn extract_all_sect_elders(world_seed: i32, d: &GameData) -> Vec<SectData> {
    let sects = [
        (1, "丹霞洞天"), (2, "昆仑宫"), (3, "极天宫"), (4, "紫霄宗"),
        (5, "玄一道"), (6, "青莲剑宗"), (7, "栖霞洞天"), (8, "百蛮山"),
        (9, "七仟坞"), (10, "七杀魔宫"), (11, "合欢派"), (12, "万妖殿"), (13, "武当派"),
    ];
    
    let mut results = Vec::with_capacity(sects.len());

    for &(sect_id, sect_name) in sects.iter() {
        let mut elders = Vec::new();
        let elder_count = 15; 

        for rlevel in 4..=5 {
            // 从 0 开始循环，0 对应动态生成的 NPC，1 及以上对应固定大能
            for index in 0..elder_count {
                // 修复：遵循 j.txt 中的原版计算公式，将 100_000 和 10_000 修正为 1000 和 100
                let localtion_seed = (sect_id * 1000) + (rlevel * 100) + index as i32;
                let base_entity_seed = world_seed.wrapping_add(localtion_seed);

                // ==========================================
                // Scope A: 基础属性与姓名 (隔离变长周期消耗)
                // ==========================================
                let mut prng_a = DotNetRandom::new(base_entity_seed);
                
                let mut age = 0.0;
                let mut num = 0;
                while (age < 14.0 || age > 60.0) && num < 100 {
                    let mut num3 = 0.0;
                    let mut num1 = 0.0;
                    loop {
                        num1 = (prng_a.next_double() * 2.0) - 1.0;
                        let num2 = (prng_a.next_double() * 2.0) - 1.0;
                        num3 = num1 * num1 + num2 * num2;
                        if num3 > 0.0 && num3 < 1.0 { break; }
                    }
                    let num4 = (-2.0 * num3.ln() / num3).sqrt();
                    let rand_normal = num1 * num4;
                    age = 22.2 + rand_normal * 9.0;
                    age = age.trunc();
                    
                    if prng_a.next_double() <= 0.1 {
                        age = prng_a.next_range_strict(1, 10) as f64;
                    }
                    num += 1;
                }
                age -= prng_a.next_double() * 0.8 + 0.1;
                
                let sex = prng_a.next_range_strict(1, 3);
                prng_a.next_range_strict(1, if sex != 1 { 6 } else { 11 }); // HairID
                prng_a.next_double(); // ScaleAdd
                
                let mut prefix = String::new();
                let mut suffix = String::new();
                if !d.names_prefix.is_empty() {
                    let prefix_idx = prng_a.next_range_strict(1, d.names_prefix.len() as i32 + 1) - 1;
                    prefix = d.names_prefix[prefix_idx as usize].clone();
                }
                
                let suffixes = if sex == 1 { &d.names_m_suffix } else { &d.names_f_suffix };
                if !suffixes.is_empty() {
                    let suffix_idx = prng_a.next_range_strict(1, suffixes.len() as i32 + 1) - 1;
                    suffix = suffixes[suffix_idx as usize].clone();
                }
                
                if sex == 1 && age > 30.0 {
                    if prng_a.next_double() <= 0.4 {
                        prng_a.next_range_strict(1, 6);
                    }
                }
                
                let name = if !prefix.is_empty() || !suffix.is_empty() {
                    format!("{}{}", prefix, suffix)
                } else {
                    let elder_type = if rlevel == 4 { "元神长老" } else { "在世真仙" };
                    format!("未解析大能 [{} #{}]", elder_type, base_entity_seed)
                };
                
                let level_name = if rlevel == 4 { "元神期 (God1)" } else { "在世真仙 (God2)" };

                // ==========================================
                // Scope B: 专属符箓 (确定性 Seed 覆写)
                // ==========================================
                let mut prng_b = DotNetRandom::new(base_entity_seed);
                let mut talismans = Vec::with_capacity(3);
                
                prng_b.advance(3); 

                for _ in 0..3 {
                    let spell_idx = prng_b.next_range_strict(0, 1.max(d.spells.len() as i32));
                    talismans.push(get_safe_item(&d.spells, spell_idx as usize));
                }

                // ==========================================
                // Scope C: 杂项物品栏 (基于索引的并行 PRNG)
                // ==========================================
                let num_scalar = (base_entity_seed.wrapping_add(8888)) as u32; 
                let inventory_matrix_size = 5; 
                let mut inventory = Vec::with_capacity(inventory_matrix_size);

                for i in 0..inventory_matrix_size {
                    let mut prng_c = DotNetRandom::new((num_scalar.wrapping_add(i as u32)) as i32);
                    let item_idx = prng_c.next_range_strict(0, 1.max(d.tools.len() as i32));
                    inventory.push(get_safe_item(&d.tools, item_idx as usize));
                }

                elders.push(ElderData { name, level_name: level_name.to_string(), talismans, inventory });
            }
        }
        
        results.push(SectData {
            sect_name: sect_name.to_string(),
            elders,
        });
    }

    results
}