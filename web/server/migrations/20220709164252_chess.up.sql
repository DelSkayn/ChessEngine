-- Add up migration script here

create type "squares" as enum (
    'a1','b1','c1','d1','e1','f1','g1','h1',
    'a2','b2','c2','d2','e2','f2','g2','h2',
    'a3','b3','c3','d3','e3','f3','g3','h3',
    'a4','b4','c4','d4','e4','f4','g4','h4',
    'a5','b5','c5','d5','e5','f5','g5','h5',
    'a6','b6','c6','d6','e6','f6','g6','h6',
    'a7','b7','c7','d7','e7','f7','g7','h7',
    'a8','b8','c8','d8','e8','f8','g8','h8'
);

create type "pieces" as enum(
    'white_king',
    'white_queen',
    'white_rook',
    'white_knight',
    'white_bishop',
    'white_pawn',
    'black_king',
    'black_queen',
    'black_rook',
    'black_knight',
    'black_bishop',
    'black_pawn'
);

create type "moves" as (
    "from" "squares",
    "to" "squares",
    "piece" "pieces" 
);


create table "position"
(
    "position_id" serial primary key,
    "fen" text not null unique,
    "name" text
);

select trigger_updated_at('"position"');

create table "move"
(
    move_id serial primary key,
    "move" "moves" not null,
    "from" integer not null,
    "to" integer not null,
    foreign key("from") references "position"("position_id"),
    foreign key("to") references "position"("position_id"),
    unique("from","to")
);

select trigger_updated_at('"move"');