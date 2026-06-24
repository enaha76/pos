-- Split login accounts from floor staff.
-- `users` = people who log in (cashier / admin). `servers` = floor staff used
-- only for attribution + roster (no login).

create table users (
  user_id  text primary key default gen_random_uuid()::text,
  name     text not null,
  pin_hash text not null,
  role     text not null default 'cashier' check (role in ('cashier', 'admin')),
  active   boolean not null default true
);

-- Seed accounts (plaintext PINs hashed to bcrypt on startup, like before).
insert into users (user_id, name, pin_hash, role) values
  ('usr_cashier', 'Caissier',     '1111', 'cashier'),
  ('usr_admin',   'Propriétaire', '9999', 'admin')
on conflict (user_id) do nothing;

-- Servers no longer log in.
alter table servers drop column pin_hash;
