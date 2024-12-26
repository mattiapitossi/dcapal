-- Custom types
create
    type public.role_status as enum ('active', 'inactive');
create
    type public.app_role as enum ('admin', 'authenticated');
create
    type public.risk_tolerance as enum ('low', 'medium', 'high');
create
    type public.data_source as enum ('yahoo');
create
    type public.investment_mode as enum ('standard', 'expert');
create
    type public.investment_goal as enum ('retirement', 'education', 'wealth_building', 'other');
create
    type public.asset_type as enum ('etf', 'stock', 'bond', 'crypto', 'other');

-- USERS
create table public.users
(
    id         uuid references auth.users             not null primary key, -- UUID from auth.users
    name       text,
    email      text                                   not null unique,
    birth_date date,
    created_at timestamp with time zone default now() not null,
    updated_at timestamp with time zone default now() not null
);

-- ROLES
create table public.roles
(
    id         bigint generated by default as identity primary key,
    name       app_role                                  not null,
    status     role_status              default 'active' not null,
    created_at timestamp with time zone default now()    not null,
    updated_at timestamp with time zone default now()    not null
);

-- USER ROLES
create table public.user_roles
(
    id         bigint generated by default as identity primary key,
    user_id    uuid references public.users on delete cascade not null,
    role_id    bigint references public.roles                 not null,
    created_at timestamp with time zone default now()         not null,
    updated_at timestamp with time zone default now()         not null,
    unique (user_id, role_id)
);

-- PERMISSIONS
create table public.permissions
(
    id         bigint generated by default as identity primary key,
    name       text                                   not null unique,
    created_at timestamp with time zone default now() not null,
    updated_at timestamp with time zone default now() not null
);

-- ROLE PERMISSIONS
create table public.role_permissions
(
    id            bigint generated by default as identity primary key,
    role_id       bigint references public.roles         not null,
    permission_id bigint references public.permissions   not null,
    created_at    timestamp with time zone default now() not null,
    updated_at    timestamp with time zone default now() not null,
    unique (role_id, permission_id)
);