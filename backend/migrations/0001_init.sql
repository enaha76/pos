-- Cashier Charge schema — mirrors src/types/domain.ts.
-- Ids are TEXT (uuid by default) so they're JSON-friendly and match the client.
-- Money is stored in integer minor units (cents).

create table settings (
  id              int primary key default 1,
  spot_label      text not null default 'Table',
  currency_symbol text not null default '$',
  constraint settings_singleton check (id = 1)
);

create table zones (
  zone_id       text primary key default gen_random_uuid()::text,
  name          text not null,
  display_order int  not null default 0,
  table_mode    text not null default 'free' check (table_mode in ('none','free','fixed')),
  spot_label    text,
  active        boolean not null default true
);

create table table_spots (
  table_id text primary key default gen_random_uuid()::text,
  zone_id  text not null references zones(zone_id),
  label    text not null,
  active   boolean not null default true
);

create table servers (
  server_id text primary key default gen_random_uuid()::text,
  name      text not null,
  pin       text not null,
  active    boolean not null default true
);

create table shifts (
  shift_id   text primary key default gen_random_uuid()::text,
  name       text not null,
  start_time text not null,
  end_time   text not null
);

create table shift_assignments (
  assignment_id text primary key default gen_random_uuid()::text,
  server_id     text not null references servers(server_id),
  zone_id       text not null references zones(zone_id),
  shift_id      text not null references shifts(shift_id),
  date          date not null
);

create table categories (
  category_id   text primary key default gen_random_uuid()::text,
  name          text not null,
  color         text not null check (color in ('blue','pink','purple','mint','amber','coral')),
  display_order int  not null default 0
);

create table products (
  product_id  text primary key default gen_random_uuid()::text,
  name        text not null,
  category_id text not null references categories(category_id),
  price       int  not null default 0,
  active      boolean not null default true
);

create table modifiers (
  modifier_id text primary key default gen_random_uuid()::text,
  product_id  text not null references products(product_id),
  name        text not null,
  price_delta int  not null default 0
);

create table reason_codes (
  reason_id text primary key default gen_random_uuid()::text,
  kind      text not null check (kind in ('void','comp','unpaid')),
  label     text not null,
  active    boolean not null default true
);

create sequence ticket_seq start 1;

create table checks (
  check_id      text primary key default gen_random_uuid()::text,
  ticket_number int  not null default nextval('ticket_seq'),
  zone_id       text not null references zones(zone_id),
  server_id     text not null references servers(server_id),
  table_id      text references table_spots(table_id),
  table_label   text,
  status        text not null default 'OPEN'
                  check (status in ('OPEN','IN_PROGRESS','CLOSED_PAID','CLOSED_UNPAID','VOIDED')),
  reason_id     text references reason_codes(reason_id),
  opened_at     timestamptz not null default now(),
  closed_at     timestamptz
);

create table order_items (
  item_id    text primary key default gen_random_uuid()::text,
  check_id   text not null references checks(check_id) on delete cascade,
  server_id  text not null references servers(server_id),
  product_id text not null references products(product_id),
  name       text not null,
  qty        int  not null default 1,
  unit_price int  not null,
  state      text not null default 'HELD' check (state in ('HELD','SENT','VOID','COMP')),
  reason_id  text references reason_codes(reason_id),
  created_at timestamptz not null default now()
);

create table payments (
  payment_id text primary key default gen_random_uuid()::text,
  check_id   text not null references checks(check_id) on delete cascade,
  method     text not null,
  amount     int  not null,
  paid_at    timestamptz not null default now()
);

create table audit_log (
  log_id    text primary key default gen_random_uuid()::text,
  actor_id  text,
  action    text not null,
  target    text not null,
  reason_id text,
  detail    text,
  timestamp timestamptz not null default now()
);

create index on checks (status);
create index on order_items (check_id);
create index on payments (check_id);
create index on shift_assignments (zone_id, shift_id, date);
