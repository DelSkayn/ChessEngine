-- Add up migration script here

create table "session" (
    session_id uuid primary key default uuid_generate_v4(),
    timestamp timestamptz not null default now(),
    user_id serial unique not null references "user"(user_id)
);

create view "session_view" as
    select 
        "session".session_id as session_id,
        "session".timestamp as timestamp,
        "session".user_id as user_id,
        "user".is_admin as is_admin
    from "session"
    inner join "user" on "session".user_id="user".user_id
