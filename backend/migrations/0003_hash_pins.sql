-- PINs are no longer stored in plaintext. Rename the column; the app hashes any
-- remaining plaintext values (e.g. the seeded demo PINs) to bcrypt on startup.
alter table servers rename column pin to pin_hash;
