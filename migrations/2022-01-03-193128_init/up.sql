-- Add migration script here
create table genre
(
    genre_id serial
        constraint genre_pk
            primary key,
    name     VARCHAR(64) not null
);

create unique index genre_name_uindex
    on genre (name);

create table artists
(
    artist_id serial
        primary key,
    name      VARCHAR(256) not null
        constraint artist_unique
            unique,
    is_faved  boolean     not null default false,
    is_known_spot boolean not null default false,
    is_known_local boolean not null default false,
    spot_id   VARCHAR(64)
);

create table artist_genre
(
    id        serial primary key,
    artist_id integer not null
        references artists (artist_id),
    genre_id  integer not null
        references genre (genre_id)
);

create table albums
(
    album_id     serial
        primary key,
    name         VARCHAR(256) not null,
    album_type   integer     not null default 0,
    year         integer     not null,
    total_tracks integer,
    is_faved     boolean     not null default false,
    is_known_spot boolean not null default false,
    is_known_local boolean not null default false,
    was_aow      boolean     not null default false,
    spot_id      VARCHAR(64)
);

create table album_artists
(
    id        serial primary key,
    album_id  integer not null
        references albums (album_id),
    artist_id integer not null
        references artists (artist_id)
);

create table tracks
(
    track_id     serial
        primary key,
    title        VARCHAR(256) not null,
    album_id     INTEGER     not null
        references albums (album_id)
            on delete cascade,
    disc_number  integer,
    track_number integer,
    duration_ms  bigint     not null,
    is_faved     boolean     not null default false,
    local_file   VARCHAR(1024),
    spot_id      VARCHAR(64)
);

create table track_artist
(
    id        serial primary key,
    track_id  integer not null
        references tracks (track_id),
    artist_id integer not null
        references artists (artist_id)
);

create table albums_of_week
(
    aow_id         serial
        primary key,
    album_id       INTEGER     not null
        references albums (album_id)
            on delete cascade,
    year           integer     not null,
    week           integer     not null,
    source_name    VARCHAR(30) not null,
    source_comment TEXT        not null,
    source_date    VARCHAR(40) not null,
    track_list_raw TEXT
);

create table charts_of_week
(
    chart_id       serial
        primary key,
    year           integer     not null,
    calendar_week  integer     not null,
    source_name    VARCHAR(30) not null,
    track_id       INTEGER     not null
        references tracks (track_id)
            on delete cascade,
    chart_position integer     not null
);

create table track_fav_proposals
(
    track_fav_id     serial primary key,
    source_name      varchar(30) not null,
    source_prop      varchar(1024) not null,
    ext_track_title  varchar(256) not null,
    ext_artist_name  varchar(256) not null,
    ext_album_name   varchar(256),
    track_id integer references tracks (track_id) on delete cascade
);