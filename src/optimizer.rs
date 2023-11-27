use crate::data::Recipe;

use super::dsp::{ComputedRecipe, RecipeRequirements};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Optimizer {
    recipe_map: HashMap<String, Vec<Recipe>>,
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            recipe_map: HashMap::new(),
        }
    }

    pub fn set_recipes(&mut self, recipes: HashMap<String, Vec<Recipe>>) {
        self.recipe_map = recipes;
    }

    #[tracing::instrument]
    fn get_recipe(&self, item_name: String, recipe_idx: i64) -> Option<Recipe> {
        let name = item_name.to_lowercase();
        let recipes = self.recipe_map.get(&name)?;

        if recipes.len() > recipe_idx as usize {
            let recipe = (recipes[recipe_idx as usize]).clone();
            return Some(recipe.clone());
        }

        Some(recipes[0].clone())
    }

    #[tracing::instrument]
    fn get_recipes(&self) -> Vec<Vec<Recipe>> {
        let mut recipes: Vec<Vec<Recipe>> = vec![];
        for (_, recipe) in self.recipe_map.iter() {
            recipes.push(recipe.to_vec());
        }
        recipes
    }

    #[tracing::instrument]
    pub fn get_optimal_recipe(
        &self,
        item_name: String,
        crafting_speed: f64,
        parent_item_name: String,
        seen_recipes: &mut HashMap<String, bool>,
        depth: i64,
        recipe_requirements: RecipeRequirements,
    ) -> Vec<ComputedRecipe> {
        let mut computed_recipes = vec![];

        if seen_recipes.contains_key(&item_name) {
            return computed_recipes;
        }
        seen_recipes.insert(item_name.clone(), true);

        let recipe_idx = recipe_requirements.get(&item_name).cloned().unwrap_or(0);
        let recipe = self.get_recipe(item_name.clone(), recipe_idx);
        if recipe.is_none() {
            return computed_recipes;
        }

        let recipe = recipe.unwrap();

        let mut consumed_mats = HashMap::new();
        let mut number_of_facilities_needed = 0_f64;
        if recipe.output_item_count > 0.0 {
            number_of_facilities_needed = recipe.time * crafting_speed / recipe.output_item_count;
        }
        for (material_name, material_count) in recipe.materials.iter() {
            let mut new_material_count = 0_f64;

            if recipe.time > 0.0 {
                new_material_count = material_count * number_of_facilities_needed / recipe.time;
            }

            consumed_mats.insert(material_name.clone(), new_material_count);
        }

        let computed_recipe: ComputedRecipe = ComputedRecipe {
            output_item: recipe.output_item.clone(),
            facility: recipe.facility.clone(),
            num_facilities_needed: number_of_facilities_needed,
            items_consumed_per_sec: consumed_mats,
            seconds_spent_per_craft: recipe.time,
            crafting_per_sec: crafting_speed,
            used_for: parent_item_name.clone(),
            depth: Some(depth),
            image: recipe.image.clone(),
        };
        computed_recipes.push(computed_recipe.clone());

        for (material_name, material_count_per_sec) in computed_recipe.items_consumed_per_sec.iter()
        {
            let target_crafting_speed = *material_count_per_sec;
            let mut seen_recipes_copy = HashMap::new();
            for (k, v) in seen_recipes.iter() {
                seen_recipes_copy.insert(k.clone(), *v);
            }
            let cr = self.get_optimal_recipe(
                material_name.clone(),
                target_crafting_speed,
                recipe.output_item.clone(),
                &mut seen_recipes_copy,
                depth + 1,
                recipe_requirements.clone(),
            );
            computed_recipes.extend(cr);
        }

        computed_recipes
    }

    #[tracing::instrument]
    fn sort_recipes(&self, recipes: &mut Vec<ComputedRecipe>) {
        recipes.sort_by(|a, b| {
            if a.depth != b.depth {
                a.depth.cmp(&b.depth)
            } else if a.output_item != b.output_item {
                a.output_item.cmp(&b.output_item)
            } else if a.used_for != b.used_for {
                a.used_for.cmp(&b.used_for)
            } else {
                a.crafting_per_sec.partial_cmp(&b.crafting_per_sec).unwrap()
            }
        });
    }

    #[tracing::instrument]
    fn combine_recipes(&self, recipes: &mut Vec<ComputedRecipe>) -> Vec<ComputedRecipe> {
        let mut unique_recipes: HashMap<String, ComputedRecipe> = HashMap::new();

        for recipe in recipes.iter() {
            if let Some(u_recipe) = unique_recipes.get_mut(&recipe.output_item) {
                let old_num = u_recipe.num_facilities_needed;
                let new_num = recipe.num_facilities_needed;
                let total_num = old_num + new_num;
                for (material_name, per_sec_consumption) in
                    u_recipe.items_consumed_per_sec.iter_mut()
                {
                    *per_sec_consumption += recipe.items_consumed_per_sec[material_name];
                }

                let mut sspc = 0_f64;
                if total_num > 0.0 {
                    sspc = (u_recipe.seconds_spent_per_craft * old_num
                        + recipe.seconds_spent_per_craft * new_num)
                        / total_num
                }
                u_recipe.seconds_spent_per_craft = sspc;
                u_recipe.crafting_per_sec += recipe.crafting_per_sec;
                u_recipe.used_for = format!(
                    "{} | {} (Uses {}/s)",
                    u_recipe.used_for, recipe.used_for, recipe.crafting_per_sec
                );
                u_recipe.num_facilities_needed += recipe.num_facilities_needed;
                u_recipe.depth = max(u_recipe.depth, recipe.depth);
            } else {
                let mut recipe = recipe.clone();
                if !recipe.used_for.is_empty() {
                    recipe.used_for =
                        format!("{} (Uses {}/s)", recipe.used_for, recipe.crafting_per_sec);
                }
                unique_recipes.insert(recipe.output_item.clone(), recipe);
            }
        }

        let mut v: Vec<ComputedRecipe> = vec![];
        for (_, value) in unique_recipes.iter() {
            v.push((*value).clone());
        }
        v
    }
}

#[tracing::instrument]
fn max(a: Option<i64>, b: Option<i64>) -> Option<i64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::data::Recipe;

    #[test]

    fn test_max() {
        assert_eq!(super::max(Some(1), Some(2)), Some(2));
        assert_eq!(super::max(Some(2), Some(1)), Some(2));
        assert_eq!(super::max(Some(1), None), Some(1));
        assert_eq!(super::max(None, Some(1)), Some(1));
        assert_eq!(super::max(None, None), None);
    }

    #[test]
    fn test_combine_recipes() {
        let mut recipes = vec![
            super::ComputedRecipe {
                output_item: "Iron Ingot".to_string(),
                facility: "Smelter".to_string(),
                num_facilities_needed: 1.0,
                items_consumed_per_sec: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("Iron Ore".to_string(), 1.0);
                    m
                },
                seconds_spent_per_craft: 1.0,
                crafting_per_sec: 1.0,
                used_for: "".to_string(),
                depth: Some(0),
                image: None,
            },
            super::ComputedRecipe {
                output_item: "Iron Ingot".to_string(),
                facility: "Smelter".to_string(),
                num_facilities_needed: 1.0,
                items_consumed_per_sec: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("Iron Ore".to_string(), 1.0);
                    m
                },
                seconds_spent_per_craft: 1.0,
                crafting_per_sec: 1.0,
                used_for: "".to_string(),
                depth: Some(0),
                image: None,
            },
        ];

        let combined_recipes = super::Optimizer::new().combine_recipes(&mut recipes);
        assert_eq!(combined_recipes.len(), 1);
        assert_eq!(combined_recipes[0].num_facilities_needed, 2.0);
        assert_eq!(
            combined_recipes[0].items_consumed_per_sec.get("Iron Ore"),
            Some(&2.0)
        );
    }

    #[test]
    fn test_sort_recipes() {
        let mut recipes = vec![
            super::ComputedRecipe {
                output_item: "Iron Ingot A".to_string(),
                facility: "Smelter".to_string(),
                num_facilities_needed: 1.0,
                items_consumed_per_sec: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("Iron Ore".to_string(), 1.0);
                    m
                },
                seconds_spent_per_craft: 1.0,
                crafting_per_sec: 1.0,
                used_for: "".to_string(),
                depth: Some(1),
                image: None,
            },
            super::ComputedRecipe {
                output_item: "Iron Ingot B".to_string(),
                facility: "Smelter".to_string(),
                num_facilities_needed: 1.0,
                items_consumed_per_sec: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("Iron Ore".to_string(), 1.0);
                    m
                },
                seconds_spent_per_craft: 1.0,
                crafting_per_sec: 1.0,
                used_for: "".to_string(),
                depth: Some(1),
                image: None,
            },
            super::ComputedRecipe {
                output_item: "Iron Ingot C".to_string(),
                facility: "Smelter".to_string(),
                num_facilities_needed: 1.0,
                items_consumed_per_sec: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("Iron Ore".to_string(), 1.0);
                    m
                },
                seconds_spent_per_craft: 1.0,
                crafting_per_sec: 1.0,
                used_for: "".to_string(),
                depth: Some(0),
                image: None,
            },
        ];

        super::Optimizer::new().sort_recipes(&mut recipes);
        assert_eq!(recipes[0].output_item, "Iron Ingot C");
        assert_eq!(recipes[1].output_item, "Iron Ingot A");
        assert_eq!(recipes[2].output_item, "Iron Ingot B");
    }

    #[test]
    fn test_get_optimal_recipe() {
        let mut recipe_map = std::collections::HashMap::new();
        recipe_map.insert(
            "Iron Ingot".to_string().to_lowercase(),
            vec![Recipe {
                output_item: "Iron Ingot".to_string(),
                output_item_count: 1.0,
                facility: "Smelter".to_string(),
                time: 1.0,
                materials: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("Iron Ore".to_string(), 1.0);
                    m
                },
                image: None,
                min_output_item_count: None,
                max_output_item_count: None,
                market_data: None,
            }],
        );
        recipe_map.insert(
            "Iron Ore".to_string().to_lowercase(),
            vec![Recipe {
                output_item: "Iron Ore".to_string(),
                output_item_count: 1.0,
                facility: "Miner".to_string(),
                time: 1.0,
                materials: std::collections::HashMap::new(),
                image: None,
                min_output_item_count: None,
                max_output_item_count: None,
                market_data: None,
            }],
        );

        let mut seen_recipes = std::collections::HashMap::new();
        let recipe_requirements = std::collections::HashMap::new();
        let mut optimizer = super::Optimizer::new();
        optimizer.set_recipes(recipe_map);
        let recipes = optimizer
            .get_optimal_recipe(
                "Iron Ingot".to_string(),
                1.0,
                "".to_string(),
                &mut seen_recipes,
                0,
                recipe_requirements,
            )
            .clone();

        assert_eq!(recipes.len(), 2);
        assert_eq!(recipes[0].output_item, "Iron Ingot");
        assert_eq!(recipes[0].num_facilities_needed, 1.0);
        assert_eq!(
            recipes[0].items_consumed_per_sec.get("Iron Ore"),
            Some(&1.0)
        );
        assert_eq!(recipes[0].seconds_spent_per_craft, 1.0);
        assert_eq!(recipes[0].crafting_per_sec, 1.0);
        assert_eq!(recipes[0].used_for, "".to_string());
        assert_eq!(recipes[0].depth, Some(0));

        assert_eq!(recipes[1].output_item, "Iron Ore");
        assert_eq!(recipes[1].num_facilities_needed, 1.0);
        assert_eq!(recipes[1].items_consumed_per_sec.len(), 0);
        assert_eq!(recipes[1].seconds_spent_per_craft, 1.0);
        assert_eq!(recipes[1].crafting_per_sec, 1.0);
        assert_eq!(recipes[1].used_for, "Iron Ingot".to_string());
        assert_eq!(recipes[1].depth, Some(1));
    }
}
