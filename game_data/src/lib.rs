use simulator::{ActionMask, Settings};

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub name: &'static str,
    pub item_level: u32,
    pub can_be_hq: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Ingredient {
    pub item_id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Recipe {
    pub recipe_level: u32,
    pub progress: u32,
    pub quality: u32,
    pub durability: i8,
    pub progress_div: u32,
    pub progress_mod: u32,
    pub quality_div: u32,
    pub quality_mod: u32,
    pub material_quality_factor: u32,
    pub ingredients: [Ingredient; 6],
}

#[derive(Debug, Clone, Copy)]
pub struct RecipeConfiguration {
    pub item_id: u32,
    pub recipe: Recipe,
    pub hq_ingredients: [u8; 6],
}

#[derive(Debug, Clone, Copy)]
pub struct CrafterConfiguration {
    pub craftsmanship: u16,
    pub control: u16,
    pub cp: u16,
    pub job_level: u8,
    pub manipulation: bool,
}

pub const LEVELS: [u32; 90] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    120, 125, 130, 133, 136, 139, 142, 145, 148, 150, 260, 265, 270, 273, 276, 279, 282, 285, 288,
    290, 390, 395, 400, 403, 406, 409, 412, 415, 418, 420, 517, 520, 525, 530, 535, 540, 545, 550,
    555, 560,
];

pub static ITEM_IDS: phf::OrderedMap<&'static str, u32> =
    include!(concat!(env!("OUT_DIR"), "/item_ids.rs"));
pub static ITEMS: phf::OrderedMap<u32, Item> = include!(concat!(env!("OUT_DIR"), "/items.rs"));
pub static RECIPES: phf::OrderedMap<u32, Recipe> =
    include!(concat!(env!("OUT_DIR"), "/recipes.rs"));

pub fn get_craftable_item_names() -> impl Iterator<Item = &'static str> {
    RECIPES
        .keys()
        .into_iter()
        .map(|item_id| ITEMS.get(item_id).unwrap().name)
}

pub fn get_ingredients(item_name: String) -> [Ingredient; 6] {
    let item_id = ITEM_IDS.get(&item_name).unwrap();
    let recipe = RECIPES.get(item_id).unwrap();
    recipe.ingredients
}

pub fn get_game_settings(
    recipe_config: RecipeConfiguration,
    crafter_config: CrafterConfiguration,
) -> Settings {
    let recipe = recipe_config.recipe;
    let mut base_progress: f64 =
        crafter_config.craftsmanship as f64 * 10.0 / recipe.progress_div as f64 + 2.0;
    let mut base_quality: f64 =
        crafter_config.control as f64 * 10.0 / recipe.quality_div as f64 + 35.0;
    if LEVELS[crafter_config.job_level as usize - 1] <= recipe.recipe_level {
        base_progress = base_progress * recipe.progress_mod as f64 / 100.0;
        base_quality = base_quality * recipe.quality_mod as f64 / 100.0;
    }
    Settings {
        max_cp: crafter_config.cp as i16,
        max_durability: recipe.durability as i16,
        max_progress: recipe.progress,
        max_quality: recipe.quality,
        base_progress: base_progress.floor() as u32,
        base_quality: base_quality.floor() as u32,
        job_level: crafter_config.job_level,
        allowed_actions: ActionMask::from_level(
            crafter_config.job_level as u32,
            crafter_config.manipulation,
        ),
    }
}
