-- Add down migration script here

-- Finally, this is a text collation that sorts text case-insensitively, useful for `UNIQUE` indexes
-- over things like usernames and emails, without needing to remember to do case-conversion.
drop collation case_insensitive;
drop function trigger_updated_at;
drop function set_updated_at;
drop extension "uuid-ossp";
