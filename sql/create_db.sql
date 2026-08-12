create table if not exists data_directory (
    id integer primary key,
    path text not null unique
);

create table if not exists backup_entry (
    id integer primary key,
    hash varchar(64) unique not null check(length(hash) = 64),
    data_directory_id integer not null,

    foreign key(data_directory_id)
        references data_directory(id)
        on delete cascade
);
