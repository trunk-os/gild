create table users (
  id varchar primary key not null,
  username varchar not null,
  realname varchar,
  email varchar,
  phone varchar,
  password blob not null
);

create table sessions (
  id varchar primary key not null,
  secret blob not null,
  expires datetime not null,
  user_id varchar not null
);
