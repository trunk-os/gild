create table users (
  user_id integer primary key autoincrement,
  username varchar not null,
  realname varchar,
  email varchar,
  phone varchar,
  password blob not null
);

create table sessions (
  session_id integer primary key autoincrement,
  secret blob not null,
  expires datetime not null,
  user_id varchar not null
);
