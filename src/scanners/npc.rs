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
                
                // 模拟 NpcRandomMechine._RandomNpc 内部的姓名、外貌随机步长。
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