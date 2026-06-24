-- Modifiers were missing from the original seed. Add the demo set.
insert into modifiers (modifier_id, product_id, name, price_delta) values
  ('m_rare',     'p_ribeye', 'Rare',         0),
  ('m_medium',   'p_ribeye', 'Medium',       0),
  ('m_welldone', 'p_ribeye', 'Well done',    0),
  ('m_cheese',   'p_burger', 'Extra cheese', 150),
  ('m_bacon',    'p_burger', 'Add bacon',    250)
on conflict (modifier_id) do nothing;
