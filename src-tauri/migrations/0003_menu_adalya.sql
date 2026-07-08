-- Real Café Adalya menu — replaces the demo seed menu.
-- Runs on both fresh and existing installs (new migration version).
-- order_items snapshot name+price and don't FK products, so past orders survive.

delete from modifiers;
delete from products;
delete from categories;

insert into categories (category_id, name, color, display_order) values
  ('cat_cafes',    'Cafés Chauds',              'amber',  1),
  ('cat_froides',  'Boissons Froides',          'blue',   2),
  ('cat_jus',      'Jus & Milkshakes',          'mint',   3),
  ('cat_petitdej', 'Petit-Déjeuner',            'pink',   4),
  ('cat_snacks',   'Sandwichs, Tacos & Pizza',  'coral',  5),
  ('cat_poulet',   'Poulet',                    'purple', 6),
  ('cat_desserts', 'Desserts',                  'pink',   7),
  ('cat_chicha',   'Narguilé / Chicha',         'blue',   8);

insert into products (product_id, name, category_id, price, active) values
  -- Cafés Chauds
  ('p_espresso',      'Espresso',        'cat_cafes', 60, 1),
  ('p_nosnos',        'Nos-Nos',         'cat_cafes', 60, 1),
  ('p_cafe_lait',     'Café au Lait',    'cat_cafes', 60, 1),
  ('p_cappuccino',    'Cappuccino',      'cat_cafes', 70, 1),
  ('p_separe',        'Séparé',          'cat_cafes', 60, 1),
  ('p_lipton_lait',   'Lipton au Lait',  'cat_cafes', 60, 1),
  ('p_lipton_citron', 'Lipton Citron',   'cat_cafes', 60, 1),
  ('p_the',           'Thé',             'cat_cafes', 60, 1),

  -- Boissons Froides
  ('p_eau',        'Eau Minérale', 'cat_froides', 20, 1),
  ('p_coca',       'Coca-Cola',    'cat_froides', 50, 1),
  ('p_sprite',     'Sprite',       'cat_froides', 50, 1),
  ('p_hawai',      'Hawai',        'cat_froides', 50, 1),
  ('p_double7',    'Double Seven', 'cat_froides', 50, 1),

  -- Jus & Milkshakes
  ('p_jus_avocat',   'Jus d''Avocat',       'cat_jus', 150, 1),
  ('p_jus_orange',   'Jus d''Orange',       'cat_jus', 150, 1),
  ('p_cocktail',     'Cocktail de Fruits',  'cat_jus', 150, 1),
  ('p_mojito',       'Mojito',              'cat_jus', 150, 1),
  ('p_ms_fraise',    'Milkshake Fraise',    'cat_jus', 150, 1),
  ('p_ms_banane',    'Milkshake Banane',    'cat_jus', 150, 1),
  ('p_ms_chocolat',  'Milkshake Chocolat',  'cat_jus', 150, 1),
  ('p_ms_mangue',    'Milkshake Mangue',    'cat_jus', 150, 1),

  -- Petit-Déjeuner
  ('p_pdj_classique', 'Petit-Déjeuner Classique', 'cat_petitdej', 160, 1),
  ('p_pdj_adalya',    'Petit-Déjeuner Adalya',    'cat_petitdej', 260, 1),

  -- Sandwichs, Tacos & Pizza
  ('p_sand_poulet',      'Sandwich Poulet',          'cat_snacks', 120, 1),
  ('p_sand_viande',      'Sandwich Viande',          'cat_snacks', 120, 1),
  ('p_sand_poulet_sp',   'Sandwich Poulet Spécial',  'cat_snacks', 150, 1),
  ('p_sand_viande_sp',   'Sandwich Viande Spécial',  'cat_snacks', 150, 1),
  ('p_tacos_poulet',     'Tacos Poulet',             'cat_snacks', 150, 1),
  ('p_tacos_viande',     'Tacos Viande',             'cat_snacks', 150, 1),
  ('p_pizza_poulet',     'Pizza Poulet',             'cat_snacks', 200, 1),
  ('p_pizza_viande',     'Pizza Viande',             'cat_snacks', 200, 1),
  ('p_pizza_margarita',  'Pizza Margarita',          'cat_snacks', 150, 1),

  -- Poulet
  ('p_poulet_quart',  '1/4 Poulet',      'cat_poulet', 150, 1),
  ('p_poulet_demi',   '1/2 Poulet',      'cat_poulet', 200, 1),
  ('p_poulet_entier', '1 Poulet entier', 'cat_poulet', 250, 1),

  -- Desserts
  ('p_crepe_nutella', 'Crêpe Nutella', 'cat_desserts',  50, 1),
  ('p_crepe_banane',  'Crêpe Banane',  'cat_desserts', 100, 1),

  -- Narguilé / Chicha
  ('p_chicha_pomme',       'Chicha Pomme',              'cat_chicha', 100, 1),
  ('p_chicha_mangue',      'Chicha Mangue',             'cat_chicha', 100, 1),
  ('p_chicha_miamor',      'Chicha Miamor',             'cat_chicha', 100, 1),
  ('p_chicha_malove',      'Chicha Ma Love',            'cat_chicha', 100, 1),
  ('p_chicha_hawai',       'Chicha Hawai',              'cat_chicha', 100, 1),
  ('p_chicha_cheikhmoni',  'Chicha Cheikh Moni',        'cat_chicha', 100, 1),
  ('p_chicha_ladyclair',   'Chicha Lady Clair',         'cat_chicha', 100, 1),
  ('p_chicha_max',         'Chicha Max (sauce menthe)', 'cat_chicha', 100, 1),
  ('p_chicha_mixpomme',    'Chicha Mix Pomme-Menthe',   'cat_chicha', 100, 1),
  ('p_chicha_doublemelon', 'Chicha Double Melon',       'cat_chicha', 100, 1);
