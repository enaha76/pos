-- Modifier groups: options sharing a non-null group are single-select (e.g. steak
-- temperature); ungrouped options are independent add-ons (e.g. extra cheese).
alter table modifiers add column mod_group text;
update modifiers set mod_group = 'Temperature' where product_id = 'p_ribeye';
