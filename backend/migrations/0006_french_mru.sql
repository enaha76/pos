-- Localize to French + MRU (Mauritanian ouguiya).

update settings set currency_symbol = 'MRU', spot_label = 'Table' where id = 1;

-- Zones
update zones set name = 'Extérieur', spot_label = 'Place' where zone_id = 'zone_out';

-- Shifts
update shifts set name = 'Matin' where shift_id = 'shift_am';
update shifts set name = 'Soir'  where shift_id = 'shift_pm';

-- Categories
update categories set name = 'Entrées'         where category_id = 'cat_starters';
update categories set name = 'Plats'           where category_id = 'cat_mains';
update categories set name = 'Accompagnements' where category_id = 'cat_sides';
update categories set name = 'Boissons'        where category_id = 'cat_drinks';
update categories set name = 'Desserts'         where category_id = 'cat_desserts';

-- Products
update products set name = 'Bruschetta'              where product_id = 'p_bruschetta';
update products set name = 'Soupe du jour'           where product_id = 'p_soup';
update products set name = 'Calamars'                where product_id = 'p_calamari';
update products set name = 'Salade verte'            where product_id = 'p_salad';
update products set name = 'Entrecôte'               where product_id = 'p_ribeye';
update products set name = 'Saumon grillé'           where product_id = 'p_salmon';
update products set name = 'Burger maison'           where product_id = 'p_burger';
update products set name = 'Pâtes à la truffe'       where product_id = 'p_pasta';
update products set name = 'Risotto aux champignons' where product_id = 'p_risotto';
update products set name = 'Poulet rôti'             where product_id = 'p_chicken';
update products set name = 'Frites'                  where product_id = 'p_fries';
update products set name = 'Légumes de saison'       where product_id = 'p_veg';
update products set name = 'Corbeille de pain'       where product_id = 'p_bread';
update products set name = 'Mojito'                  where product_id = 'p_mojito';
update products set name = 'Vin maison'              where product_id = 'p_wine';
update products set name = 'Bière pression'          where product_id = 'p_beer';
update products set name = 'Soda'                    where product_id = 'p_soda';
update products set name = 'Café'                    where product_id = 'p_coffee';
update products set name = 'Eau gazeuse'             where product_id = 'p_water';
update products set name = 'Tiramisu'                where product_id = 'p_tiramisu';
update products set name = 'Cheesecake'              where product_id = 'p_cheesecake';
update products set name = 'Glace'                   where product_id = 'p_icecream';

-- Modifiers
update modifiers set mod_group = 'Cuisson' where mod_group = 'Temperature';
update modifiers set name = 'Saignant'            where modifier_id = 'm_rare';
update modifiers set name = 'À point'             where modifier_id = 'm_medium';
update modifiers set name = 'Bien cuit'           where modifier_id = 'm_welldone';
update modifiers set name = 'Supplément fromage'  where modifier_id = 'm_cheese';
update modifiers set name = 'Ajouter bacon'       where modifier_id = 'm_bacon';

-- Reason codes
update reason_codes set label = 'Erreur de commande'    where reason_id = 'r_v_mistake';
update reason_codes set label = 'Client a changé d''avis' where reason_id = 'r_v_changed';
update reason_codes set label = 'Saisie en double'        where reason_id = 'r_v_duplicate';
update reason_codes set label = 'Erreur cuisine'          where reason_id = 'r_c_kitchen';
update reason_codes set label = 'Insatisfaction client'   where reason_id = 'r_c_dissatisfied';
update reason_codes set label = 'Renversé / accident'     where reason_id = 'r_c_spill';
update reason_codes set label = 'Repas du personnel'      where reason_id = 'r_c_staff';
update reason_codes set label = 'Départ sans payer'       where reason_id = 'r_u_walkout';
update reason_codes set label = 'Annulation manager'      where reason_id = 'r_u_writeoff';
