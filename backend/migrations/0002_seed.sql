-- Seed data — mirrors src/data/seed.ts so the API starts with usable content.

insert into settings (id, spot_label, currency_symbol)
values (1, 'Table', '$') on conflict (id) do nothing;

insert into zones (zone_id, name, display_order, table_mode, spot_label, active) values
  ('zone_vip1', 'VIP1', 1, 'fixed', null, true),
  ('zone_vip2', 'VIP2', 2, 'free', null, true),
  ('zone_out',  'Outside', 3, 'none', 'Spot', true)
on conflict (zone_id) do nothing;

insert into table_spots (table_id, zone_id, label, active) values
  ('t_v1_1', 'zone_vip1', '1', true),
  ('t_v1_2', 'zone_vip1', '2', true),
  ('t_v1_3', 'zone_vip1', '3', true),
  ('t_v1_4', 'zone_vip1', '4', true)
on conflict (table_id) do nothing;

insert into servers (server_id, name, pin, active) values
  ('srv_amina', 'Amina', '1111', true),
  ('srv_bilal', 'Bilal', '2222', true),
  ('srv_cara',  'Cara',  '3333', true)
on conflict (server_id) do nothing;

insert into shifts (shift_id, name, start_time, end_time) values
  ('shift_am', 'Morning', '07:00', '16:00'),
  ('shift_pm', 'Evening', '16:00', '01:00')
on conflict (shift_id) do nothing;

insert into categories (category_id, name, color, display_order) values
  ('cat_starters', 'Starters', 'blue',   1),
  ('cat_mains',    'Mains',    'purple', 2),
  ('cat_sides',    'Sides',    'amber',  3),
  ('cat_drinks',   'Drinks',   'mint',   4),
  ('cat_desserts', 'Desserts', 'pink',   5)
on conflict (category_id) do nothing;

insert into products (product_id, name, category_id, price, active) values
  ('p_bruschetta', 'Bruschetta',       'cat_starters', 850,  true),
  ('p_soup',       'Soup of the Day',  'cat_starters', 700,  true),
  ('p_calamari',   'Calamari',         'cat_starters', 1150, true),
  ('p_salad',      'Garden Salad',     'cat_starters', 900,  true),
  ('p_ribeye',     'Ribeye Steak',     'cat_mains',    2800, true),
  ('p_salmon',     'Grilled Salmon',   'cat_mains',    2200, true),
  ('p_burger',     'House Burger',     'cat_mains',    1600, true),
  ('p_pasta',      'Truffle Pasta',    'cat_mains',    1900, true),
  ('p_risotto',    'Mushroom Risotto', 'cat_mains',    1750, true),
  ('p_chicken',    'Roast Chicken',    'cat_mains',    1850, true),
  ('p_fries',      'Fries',            'cat_sides',    500,  true),
  ('p_veg',        'Seasonal Veg',     'cat_sides',    600,  true),
  ('p_bread',      'Bread Basket',     'cat_sides',    400,  true),
  ('p_mojito',     'Mojito',           'cat_drinks',   1100, true),
  ('p_wine',       'House Wine',       'cat_drinks',   950,  true),
  ('p_beer',       'Draft Beer',       'cat_drinks',   750,  true),
  ('p_soda',       'Soft Drink',       'cat_drinks',   350,  true),
  ('p_coffee',     'Coffee',           'cat_drinks',   400,  true),
  ('p_water',      'Sparkling Water',  'cat_drinks',   300,  true),
  ('p_tiramisu',   'Tiramisu',         'cat_desserts', 850,  true),
  ('p_cheesecake', 'Cheesecake',       'cat_desserts', 800,  true),
  ('p_icecream',   'Ice Cream',        'cat_desserts', 600,  true)
on conflict (product_id) do nothing;

insert into reason_codes (reason_id, kind, label, active) values
  ('r_v_mistake',      'void', 'Order mistake',           true),
  ('r_v_changed',      'void', 'Customer changed mind',   true),
  ('r_v_duplicate',    'void', 'Duplicate entry',         true),
  ('r_c_kitchen',      'comp', 'Kitchen error',           true),
  ('r_c_dissatisfied', 'comp', 'Customer dissatisfaction',true),
  ('r_c_spill',        'comp', 'Spill / accident',        true),
  ('r_c_staff',        'comp', 'Staff meal',              true),
  ('r_u_walkout',      'unpaid','Walkout',                true),
  ('r_u_writeoff',     'unpaid','Manager write-off',      true)
on conflict (reason_id) do nothing;

-- Roster for today (same zone, different server per shift).
insert into shift_assignments (assignment_id, server_id, zone_id, shift_id, date) values
  ('asn_1', 'srv_amina', 'zone_vip1', 'shift_am', current_date),
  ('asn_2', 'srv_bilal', 'zone_vip2', 'shift_am', current_date),
  ('asn_3', 'srv_cara',  'zone_out',  'shift_am', current_date),
  ('asn_4', 'srv_bilal', 'zone_vip1', 'shift_pm', current_date),
  ('asn_5', 'srv_cara',  'zone_vip2', 'shift_pm', current_date),
  ('asn_6', 'srv_amina', 'zone_out',  'shift_pm', current_date)
on conflict (assignment_id) do nothing;
