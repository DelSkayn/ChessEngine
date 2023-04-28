-- Add up migration script here

create table "user"
(
    user_id serial primary key,
    username text collate "case_insensitive" unique not null,

    password text not null,

    created_at timestamptz not null default now(),
    updated_at timestamptz,
    
    is_admin boolean not null
);

select trigger_updated_at('"user"');
