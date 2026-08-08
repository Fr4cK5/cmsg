create table if not exists data_directory (
    id integer primary key autoincrement,
    path text not null unique
);

create table if not exists backup_entry (
    id integer primary key autoincrement,
    digest varchar(64) unique not null check(length(digest) = 64),
    data_directory_id integer not null,

    foreign key(data_directory_id) references data_directory(id)
);
