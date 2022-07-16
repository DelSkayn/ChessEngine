-- Add up migration script here

create table "engine"
(
    engine_id serial primary key,
    name text not null,
    description text,

    engine_file text not null,

    previous_version serial references "engine"(engine_id),

    games_played integer not null default 0,
    elo double precision not null default 1000,

    uploaded_by serial not null references "user"(user_id)
);


select trigger_updated_at('"engine"');

create type "game_outcome" as enum (
    'white_won',
    'black_won',
    'drawn',
    'canceled'
);

create table "game"
(
    game_id serial primary key,

    player_white serial not null references "engine"(engine_id),
    player_black serial not null references "engine"(engine_id),

    outcome "game_outcome",

    white_elo_change double precision,
    black_elo_change double precision
);

select trigger_updated_at('"game"');

create table "game_move"
(
    game_id serial not null references "game"(game_id),
    move_id serial not null references "move"(move_id),

    white_move boolean not null,
    move_count integer not null,

    time_taken bigint not null,

    primary key(game_id,move_id)
);

select trigger_updated_at('"game_move"');
