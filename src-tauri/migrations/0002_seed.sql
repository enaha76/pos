-- Seed data (French + MRU). User PINs are seeded in plaintext and hashed to
-- SHA-256 by the app on first launch (see src/lib/apiSqlite.ts).

insert into settings (id, spot_label, currency_symbol) values (1, 'Table', 'MRU');

insert into zones (zone_id, name, display_order, table_mode, spot_label, active) values
  ('zone_vip1', 'VIP1', 1, 'fixed', null, 1),
  ('zone_vip2', 'VIP2', 2, 'free', null, 1),
  ('zone_out',  'Extérieur', 3, 'none', 'Place', 1);

insert into table_spots (table_id, zone_id, label, active) values
  ('t_v1_1', 'zone_vip1', '1', 1),
  ('t_v1_2', 'zone_vip1', '2', 1),
  ('t_v1_3', 'zone_vip1', '3', 1),
  ('t_v1_4', 'zone_vip1', '4', 1);

insert into servers (server_id, name, active) values
  ('srv_amina', 'Amina', 1),
  ('srv_bilal', 'Bilal', 1),
  ('srv_cara',  'Cara', 1);

insert into shifts (shift_id, name, start_time, end_time) values
  ('shift_am', 'Matin', '07:00', '16:00'),
  ('shift_pm', 'Soir', '16:00', '01:00');

-- Roster for the day of first launch (date('now') keeps today non-empty).
insert into shift_assignments (assignment_id, server_id, zone_id, shift_id, date) values
  ('asn_1', 'srv_amina', 'zone_vip1', 'shift_am', date('now')),
  ('asn_2', 'srv_bilal', 'zone_vip2', 'shift_am', date('now')),
  ('asn_3', 'srv_cara',  'zone_out',  'shift_am', date('now')),
  ('asn_4', 'srv_bilal', 'zone_vip1', 'shift_pm', date('now')),
  ('asn_5', 'srv_cara',  'zone_vip2', 'shift_pm', date('now')),
  ('asn_6', 'srv_amina', 'zone_out',  'shift_pm', date('now'));

insert into categories (category_id, name, color, display_order) values
  ('cat_starters', 'Entrées', 'blue', 1),
  ('cat_mains',    'Plats', 'purple', 2),
  ('cat_sides',    'Accompagnements', 'amber', 3),
  ('cat_drinks',   'Boissons', 'mint', 4),
  ('cat_desserts', 'Desserts', 'pink', 5);

insert into products (product_id, name, category_id, price, active) values
  ('p_bruschetta', 'Bruschetta', 'cat_starters', 850, 1),
  ('p_soup', 'Soupe du jour', 'cat_starters', 700, 1),
  ('p_calamari', 'Calamars', 'cat_starters', 1150, 1),
  ('p_salad', 'Salade verte', 'cat_starters', 900, 1),
  ('p_ribeye', 'Entrecôte', 'cat_mains', 2800, 1),
  ('p_salmon', 'Saumon grillé', 'cat_mains', 2200, 1),
  ('p_burger', 'Burger maison', 'cat_mains', 1600, 1),
  ('p_pasta', 'Pâtes à la truffe', 'cat_mains', 1900, 1),
  ('p_risotto', 'Risotto aux champignons', 'cat_mains', 1750, 1),
  ('p_chicken', 'Poulet rôti', 'cat_mains', 1850, 1),
  ('p_fries', 'Frites', 'cat_sides', 500, 1),
  ('p_veg', 'Légumes de saison', 'cat_sides', 600, 1),
  ('p_bread', 'Corbeille de pain', 'cat_sides', 400, 1),
  ('p_mojito', 'Mojito', 'cat_drinks', 1100, 1),
  ('p_wine', 'Vin maison', 'cat_drinks', 950, 1),
  ('p_beer', 'Bière pression', 'cat_drinks', 750, 1),
  ('p_soda', 'Soda', 'cat_drinks', 350, 1),
  ('p_coffee', 'Café', 'cat_drinks', 400, 1),
  ('p_water', 'Eau gazeuse', 'cat_drinks', 300, 1),
  ('p_tiramisu', 'Tiramisu', 'cat_desserts', 850, 1),
  ('p_cheesecake', 'Cheesecake', 'cat_desserts', 800, 1),
  ('p_icecream', 'Glace', 'cat_desserts', 600, 1);

insert into modifiers (modifier_id, product_id, name, price_delta, mod_group) values
  ('m_rare', 'p_ribeye', 'Saignant', 0, 'Cuisson'),
  ('m_medium', 'p_ribeye', 'À point', 0, 'Cuisson'),
  ('m_welldone', 'p_ribeye', 'Bien cuit', 0, 'Cuisson'),
  ('m_cheese', 'p_burger', 'Supplément fromage', 150, null),
  ('m_bacon', 'p_burger', 'Ajouter bacon', 250, null);

insert into reason_codes (reason_id, kind, label, active) values
  ('r_v_mistake', 'void', 'Erreur de commande', 1),
  ('r_v_changed', 'void', 'Client a changé d''avis', 1),
  ('r_v_duplicate', 'void', 'Saisie en double', 1),
  ('r_c_kitchen', 'comp', 'Erreur cuisine', 1),
  ('r_c_dissatisfied', 'comp', 'Insatisfaction client', 1),
  ('r_c_spill', 'comp', 'Renversé / accident', 1),
  ('r_c_staff', 'comp', 'Repas du personnel', 1),
  ('r_u_walkout', 'unpaid', 'Départ sans payer', 1),
  ('r_u_writeoff', 'unpaid', 'Annulation manager', 1);

-- PINs in plaintext here; hashed to SHA-256 on first app launch.
insert into users (user_id, name, pin_hash, role, active) values
  ('usr_cashier', 'Caissier', '1111', 'cashier', 1),
  ('usr_admin', 'Propriétaire', '9999', 'admin', 1);
